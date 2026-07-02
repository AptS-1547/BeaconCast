import enAdmin from "@/i18n/locales/en-US/admin.json";
import enErrors from "@/i18n/locales/en-US/errors.json";
import enPublicStatus from "@/i18n/locales/en-US/publicStatus.json";
import enShell from "@/i18n/locales/en-US/shell.json";
import zhAdmin from "@/i18n/locales/zh-CN/admin.json";
import zhErrors from "@/i18n/locales/zh-CN/errors.json";
import zhPublicStatus from "@/i18n/locales/zh-CN/publicStatus.json";
import zhShell from "@/i18n/locales/zh-CN/shell.json";

export const resources = {
	"en-US": {
		shell: enShell,
		publicStatus: enPublicStatus,
		admin: enAdmin,
		errors: enErrors,
	},
	"zh-CN": {
		shell: zhShell,
		publicStatus: zhPublicStatus,
		admin: zhAdmin,
		errors: zhErrors,
	},
} as const;
