import { useCallback, useState } from "react";
import type { BootstrapDto, LiveOverview } from "../../../dto";
import type { usePreferences } from "../../../hooks/use-preferences";
import { useSettings } from "../../../hooks/use-settings";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { AboutSection } from "./about-section";
import { AppearanceSection } from "./appearance-section";
import { ConnectionSection } from "./connection-section";
import { DangerSection } from "./danger-section";
import { DataSection } from "./data-section";
import { OperationProgressView } from "./operation-progress";

type SettingsSection = "appearance" | "connection" | "data" | "about" | "danger";

const SECTIONS: SettingsSection[] = ["appearance", "connection", "data", "about", "danger"];

export function SettingsPage({
  locale,
  boot,
  overview,
  preferences
}: {
  locale: UiLocale;
  boot: BootstrapDto;
  overview: LiveOverview;
  preferences: ReturnType<typeof usePreferences>;
}) {
  const [section, setSection] = useState<SettingsSection>("connection");
  const settings = useSettings(locale, boot);
  const loadSecret = settings.loadSecret;
  const loadAbout = settings.loadAbout;
  const loadDataDir = settings.loadDataDir;
  const refreshCollector = settings.refreshCollector;
  const onEnterConnection = useCallback(() => {
    void loadSecret();
    void refreshCollector();
  }, [loadSecret, refreshCollector]);
  const onLoadAbout = useCallback(
    (force: boolean) => {
      void loadAbout(force);
    },
    [loadAbout]
  );
  const onLoadData = useCallback(() => {
    void loadDataDir();
  }, [loadDataDir]);

  return (
    <section className="flex h-full min-h-0 flex-col gap-4">
      <header className="flex items-start justify-between gap-3">
        <div>
          <h1 className="text-lg font-semibold">{t(locale, "settings.title")}</h1>
          <p className="text-sm text-muted-foreground">{t(locale, "settings.help")}</p>
        </div>
        <span className="text-sm text-muted-foreground">
          {SECTIONS.length} {t(locale, "settings.sections")}
        </span>
      </header>
      <div className="flex min-h-0 flex-1 flex-col gap-4 lg:flex-row">
        <nav
          className="flex gap-2 overflow-x-auto lg:w-52 lg:flex-col lg:overflow-visible"
          aria-label={t(locale, "settings.nav_aria")}
        >
          {SECTIONS.map((id) => (
            <button
              key={id}
              type="button"
              aria-current={section === id ? "page" : undefined}
              className={cn(
                "rounded-md px-3 py-2 text-left text-sm hover:bg-muted/40",
                section === id && "bg-primary text-primary-foreground"
              )}
              onClick={() => setSection(id)}
            >
              <span className="block">{t(locale, `settings.section.${id}`)}</span>
              <span className={cn("block text-xs", section === id ? "opacity-80" : "text-muted-foreground")}>
                {t(locale, `settings.section.${id === "appearance" ? "appearance_hint" : id === "connection" ? "connection_hint" : id === "data" ? "data_hint" : id === "about" ? "about_hint" : "danger_hint"}`)}
              </span>
            </button>
          ))}
        </nav>
        <div className="min-h-0 min-w-0 flex-1 overflow-auto">
          {section === "appearance" ? (
            <AppearanceSection
              locale={locale}
              theme={preferences.prefs.theme}
              font={preferences.prefs.font}
              fontSize={preferences.prefs.fontSize}
              density={preferences.prefs.density}
              fonts={preferences.fonts}
              fontsError={preferences.fontsError}
              sidebarWidth={preferences.prefs.sidebarWidth}
              onLocale={(next) => void preferences.setLocale(next)}
              onTheme={(next) => void preferences.setTheme(next)}
              onFont={(next) => void preferences.setFont(next)}
              onFontSize={(next) => void preferences.setFontSize(next)}
              onDensity={(next) => void preferences.setDensity(next)}
              onSidebarWidth={(width) => void preferences.commitSidebarWidth(width)}
            />
          ) : null}
          {section === "connection" ? (
            <ConnectionSection
              locale={locale}
              address={settings.address}
              targets={settings.targets}
              secret={settings.secret}
              settings={settings.settings}
              session={overview.health.session}
              collectorRunning={settings.collectorRunning}
              probeMessage={settings.probe.messageZh}
              probeState={settings.probe.state}
              wizardComplete={boot.wizardComplete}
              onAddress={settings.setAddress}
              onTargets={settings.setTargets}
              onSecret={settings.setSecret}
              onSave={() => void settings.saveConnection()}
              onTest={() => void settings.testConnection()}
              onDisconnect={() => void settings.disconnect()}
              onReconnect={() => void settings.reconnect()}
              onPause={() => void settings.pauseCollector()}
              onResume={() => void settings.resumeCollector()}
              onCompleteWizard={() => void settings.completeWizard()}
              onEnter={onEnterConnection}
            />
          ) : null}
          {section === "data" ? (
            <DataSection
              locale={locale}
              logDir={boot.logDir ?? ""}
              dataDir={settings.dataDir}
              retention={settings.retention}
              onLoad={onLoadData}
              onOpenLog={() => void settings.openLogDir()}
              onPreviewRetention={() => void settings.previewRetention()}
              onRunRetention={() => void settings.runRetention()}
              onBackup={() => void settings.createBackup()}
              onRestore={() => void settings.restoreBackup()}
              onValidate={() => settings.validateBackup()}
              onVacuum={() => void settings.vacuum()}
            />
          ) : null}
          {section === "about" ? (
            <AboutSection
              locale={locale}
              about={settings.about}
              loading={settings.aboutLoading}
              error={settings.aboutError}
              onLoad={onLoadAbout}
              onOpenReleases={() => void settings.openReleases()}
            />
          ) : null}
          {section === "danger" ? (
            <DangerSection
              locale={locale}
              preview={settings.deletePreview}
              report={settings.deleteReport}
              onPreview={() => void settings.previewDelete()}
              onConfirm={(phrase) => void settings.confirmDelete(phrase)}
            />
          ) : null}
          <div className="mt-4">
            <OperationProgressView
              locale={locale}
              progress={settings.progress}
              onCancel={() => void settings.cancelOperation()}
            />
          </div>
          {settings.errorZh ? (
            <p className="mt-3 text-sm text-destructive" role="alert">
              {settings.errorZh}
            </p>
          ) : null}
        </div>
      </div>
    </section>
  );
}
