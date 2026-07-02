//! Read-time activity classification projection.

use std::collections::HashMap;

use sea_orm::ConnectionTrait;

use crate::db::repository::beacon_repo;
use crate::errors::Result;
use crate::types::ActivityInferenceSource;

const IDLE_STATUS: &str = "idle";
const IDLE_ACTIVITY_KIND: &str = "idle";
const IDLE_CATEGORY: &str = "idle";
const UNCLASSIFIED_STATUS: &str = "unclassified";
const UNCLASSIFIED_ACTIVITY_KIND: &str = "unclassified_activity";
const UNCLASSIFIED_CATEGORY: &str = "unclassified";

#[derive(Debug, Clone)]
pub struct ActivityObservation<'a> {
    pub idle: bool,
    pub app_label: Option<&'a str>,
}

impl<'a> ActivityObservation<'a> {
    pub fn from_event(event: &'a crate::entities::activity_event::Model) -> Self {
        Self {
            idle: event.idle,
            app_label: event.app_label.as_deref(),
        }
    }

    pub fn from_usage_span(span: &'a crate::entities::activity_usage_span::Model) -> Self {
        Self {
            idle: span.idle,
            app_label: span.app_label.as_deref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectedActivity {
    pub status: String,
    pub category: Option<String>,
    pub activity_kind: String,
    pub inference_source: ActivityInferenceSource,
    pub application_key: Option<String>,
    pub application_label: Option<String>,
    pub action_key: Option<String>,
    pub action_label: Option<String>,
    pub action_public_label: Option<String>,
    pub message_template: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ActivityApplicationIdentity<'a> {
    pub key: &'a str,
    pub label: &'a str,
}

impl ProjectedActivity {
    pub fn application_identity<'a>(
        &'a self,
        observed_app_label: Option<&'a str>,
    ) -> Option<ActivityApplicationIdentity<'a>> {
        if let Some(label) = self.application_label.as_deref() {
            return Some(ActivityApplicationIdentity {
                key: self.application_key.as_deref().unwrap_or(label),
                label,
            });
        }
        observed_app_label.map(|label| ActivityApplicationIdentity { key: label, label })
    }
}

#[derive(Debug, Clone)]
pub struct ActivityProjectionContext {
    app_rules: HashMap<String, ActivityProjectionRule>,
}

#[derive(Debug, Clone)]
struct ActivityProjectionRule {
    application_key: String,
    application_label: String,
    action_key: String,
    action_label: String,
    status: String,
    category: String,
    public_label: String,
    message_template: String,
}

impl ActivityProjectionContext {
    pub async fn load<C: ConnectionTrait>(db: &C) -> Result<Self> {
        let index = beacon_repo::load_activity_classification_index(db).await?;
        Ok(Self::from_index(index))
    }

    pub fn project(&self, observation: ActivityObservation<'_>) -> ProjectedActivity {
        if observation.idle {
            return fallback(
                IDLE_STATUS,
                IDLE_ACTIVITY_KIND,
                IDLE_CATEGORY,
                ActivityInferenceSource::IdleDetector,
            );
        }
        if let Some(app_label) = observation.app_label {
            let normalized = beacon_repo::normalize_app_alias(app_label);
            if let Some(rule) = self.app_rules.get(&normalized) {
                return ProjectedActivity {
                    status: rule.status.clone(),
                    category: Some(rule.category.clone()),
                    activity_kind: rule.action_key.clone(),
                    inference_source: ActivityInferenceSource::ServerRule,
                    application_key: Some(rule.application_key.clone()),
                    application_label: Some(rule.application_label.clone()),
                    action_key: Some(rule.action_key.clone()),
                    action_label: Some(rule.action_label.clone()),
                    action_public_label: Some(rule.public_label.clone()),
                    message_template: Some(rule.message_template.clone()),
                };
            }
        }
        fallback(
            UNCLASSIFIED_STATUS,
            UNCLASSIFIED_ACTIVITY_KIND,
            UNCLASSIFIED_CATEGORY,
            ActivityInferenceSource::ServerRule,
        )
    }

    pub fn project_event(
        &self,
        event: &crate::entities::activity_event::Model,
    ) -> ProjectedActivity {
        self.project(ActivityObservation::from_event(event))
    }

    pub fn project_usage_span(
        &self,
        span: &crate::entities::activity_usage_span::Model,
    ) -> ProjectedActivity {
        self.project(ActivityObservation::from_usage_span(span))
    }

    fn from_index(index: beacon_repo::ActivityClassificationIndex) -> Self {
        let actions = index
            .actions
            .into_iter()
            .filter(|action| action.enabled)
            .map(|action| (action.id, action))
            .collect::<HashMap<_, _>>();
        let applications = index
            .applications
            .into_iter()
            .filter(|application| application.enabled)
            .map(|application| (application.id, application))
            .collect::<HashMap<_, _>>();

        let mut app_rules = HashMap::new();
        for alias in index.aliases {
            let Some(application) = applications.get(&alias.application_id) else {
                continue;
            };
            let Some(action_id) = application.default_action_id else {
                continue;
            };
            let Some(action) = actions.get(&action_id) else {
                continue;
            };
            app_rules.insert(
                alias.normalized_alias,
                ActivityProjectionRule {
                    application_key: application.app_key.clone(),
                    application_label: application.display_name.clone(),
                    action_key: action.action_key.clone(),
                    action_label: action.label.clone(),
                    status: action.status.clone(),
                    category: action.category.clone(),
                    public_label: action.public_label.clone(),
                    message_template: action.message_template.clone(),
                },
            );
        }

        Self { app_rules }
    }
}

fn fallback(
    status: &str,
    activity_kind: &str,
    category: &str,
    inference_source: ActivityInferenceSource,
) -> ProjectedActivity {
    ProjectedActivity {
        status: status.to_string(),
        category: Some(category.to_string()),
        activity_kind: activity_kind.to_string(),
        inference_source,
        application_key: None,
        application_label: None,
        action_key: None,
        action_label: None,
        action_public_label: None,
        message_template: None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::entities::{activity_action, activity_application, activity_application_alias};

    #[test]
    fn projection_matches_enabled_application_alias_to_action() {
        let context = context_with(
            vec![action(1, "writing_code", true)],
            vec![application(10, Some(1), true)],
            vec![alias(10, "Visual Studio Code", "visual studio code")],
        );

        let projected = context.project(ActivityObservation {
            idle: false,
            app_label: Some("  Visual   Studio Code  "),
        });

        assert_eq!(projected.status, "coding");
        assert_eq!(projected.category.as_deref(), Some("coding"));
        assert_eq!(projected.activity_kind, "writing_code");
        assert_eq!(
            projected.inference_source,
            ActivityInferenceSource::ServerRule
        );
        assert_eq!(projected.action_key.as_deref(), Some("writing_code"));
        assert_eq!(projected.application_key.as_deref(), Some("app-10"));
        assert_eq!(projected.application_label.as_deref(), Some("App 10"));
        assert_eq!(
            projected.action_public_label.as_deref(),
            Some("Writing code")
        );
    }

    #[test]
    fn idle_projection_takes_precedence_over_app_rule() {
        let context = context_with(
            vec![action(1, "writing_code", true)],
            vec![application(10, Some(1), true)],
            vec![alias(10, "Code", "code")],
        );

        let projected = context.project(ActivityObservation {
            idle: true,
            app_label: Some("Code"),
        });

        assert_eq!(projected.status, "idle");
        assert_eq!(projected.activity_kind, "idle");
        assert_eq!(
            projected.inference_source,
            ActivityInferenceSource::IdleDetector
        );
        assert!(projected.action_key.is_none());
    }

    #[test]
    fn disabled_action_or_application_does_not_project() {
        for (actions, applications) in [
            (
                vec![action(1, "writing_code", false)],
                vec![application(10, Some(1), true)],
            ),
            (
                vec![action(1, "writing_code", true)],
                vec![application(10, Some(1), false)],
            ),
        ] {
            let context = context_with(actions, applications, vec![alias(10, "Code", "code")]);

            let projected = context.project(ActivityObservation {
                idle: false,
                app_label: Some("Code"),
            });

            assert_eq!(projected.status, "unclassified");
            assert_eq!(projected.activity_kind, "unclassified_activity");
            assert!(projected.action_key.is_none());
        }
    }

    fn context_with(
        actions: Vec<activity_action::Model>,
        applications: Vec<activity_application::Model>,
        aliases: Vec<activity_application_alias::Model>,
    ) -> ActivityProjectionContext {
        ActivityProjectionContext::from_index(beacon_repo::ActivityClassificationIndex {
            actions,
            applications,
            aliases,
        })
    }

    fn action(id: i64, key: &str, enabled: bool) -> activity_action::Model {
        let now = Utc::now();
        activity_action::Model {
            id,
            action_key: key.to_string(),
            label: "Writing code".to_string(),
            status: "coding".to_string(),
            category: "coding".to_string(),
            public_label: "Writing code".to_string(),
            message_template: "{action}".to_string(),
            enabled,
            sort_order: 10,
            created_at: now,
            updated_at: now,
        }
    }

    fn application(
        id: i64,
        default_action_id: Option<i64>,
        enabled: bool,
    ) -> activity_application::Model {
        let now = Utc::now();
        activity_application::Model {
            id,
            app_key: format!("app-{id}"),
            display_name: format!("App {id}"),
            default_action_id,
            enabled,
            created_at: now,
            updated_at: now,
        }
    }

    fn alias(
        application_id: i64,
        alias: &str,
        normalized_alias: &str,
    ) -> activity_application_alias::Model {
        activity_application_alias::Model {
            id: application_id,
            application_id,
            alias: alias.to_string(),
            normalized_alias: normalized_alias.to_string(),
            created_at: Utc::now(),
        }
    }
}
