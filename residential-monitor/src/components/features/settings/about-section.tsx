import { useEffect, useRef } from "react";
import type { AboutDto } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

function Row({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <dt className="text-muted-foreground">{label}</dt>
      <dd className={mono ? "font-mono break-all" : undefined}>{value}</dd>
    </div>
  );
}

export function AboutSection({
  locale,
  about,
  loading,
  error,
  onLoad,
  onOpenReleases
}: {
  locale: UiLocale;
  about: AboutDto | null;
  loading: boolean;
  error: string;
  onLoad: (force: boolean) => void;
  onOpenReleases: () => void;
}) {
  const urlRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    onLoad(false);
  }, [onLoad]);

  const selectUrl = (): void => {
    const el = urlRef.current;
    if (!el) {
      return;
    }
    el.scrollIntoView({ block: "nearest" });
    const selection = window.getSelection();
    if (!selection) {
      return;
    }
    const range = document.createRange();
    range.selectNodeContents(el);
    selection.removeAllRanges();
    selection.addRange(range);
  };

  return (
    <section className="space-y-4" id="about">
      <div>
        <h2 className="text-base font-semibold">{t(locale, "settings.about")}</h2>
        <p className="text-sm text-muted-foreground">{t(locale, "settings.about_help")}</p>
      </div>
      {about ? (
        <dl className="grid gap-2 text-sm sm:grid-cols-2">
          <Row label={t(locale, "settings.about_label.product")} value={about.productName} />
          <Row label={t(locale, "settings.about_label.version")} value={about.version} mono />
          <Row label={t(locale, "settings.about_label.binary")} value={about.binaryName} mono />
          <Row label={t(locale, "settings.about_label.identifier")} value={about.identifier} mono />
          <Row label={t(locale, "settings.about_label.aumid")} value={about.aumid} mono />
          <div className="sm:col-span-2">
            <dt className="text-muted-foreground">{t(locale, "settings.about_label.signature")}</dt>
            <dd>
              {about.signed ? t(locale, "settings.signed") : t(locale, "settings.unsigned")}
              <p className="text-xs text-muted-foreground">{about.signatureNoteZh}</p>
            </dd>
          </div>
          <Row
            label={t(locale, "settings.about_label.updater")}
            value={about.updaterPlugin ? t(locale, "settings.about_updater_on") : t(locale, "settings.about_updater_off")}
          />
          <Row
            label={t(locale, "settings.about_label.service")}
            value={about.windowsService ? t(locale, "settings.about_service_on") : t(locale, "settings.about_service_off")}
          />
          <Row label={t(locale, "settings.about_label.license")} value={t(locale, "settings.about_license_value")} />
          <Row label={t(locale, "settings.about_label.platform")} value={t(locale, "settings.about_platform_value")} />
          <Row label={t(locale, "settings.about_label.privacy")} value={t(locale, "settings.about_privacy_value")} />
          <div className="sm:col-span-2">
            <dt className="text-muted-foreground">{t(locale, "settings.about_label.releases")}</dt>
            <dd>
              <span ref={urlRef} className="font-mono break-all">
                {about.releasesUrl}
              </span>
            </dd>
          </div>
        </dl>
      ) : error && !loading ? (
        <p className="text-sm text-destructive" role="alert">
          {error}
        </p>
      ) : (
        <p className="text-sm text-muted-foreground" role="status">
          {t(locale, "settings.about_loading")}
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" disabled={loading} onClick={() => onLoad(true)}>
          {t(locale, "settings.refresh_about")}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={loading}
          onClick={() => {
            onOpenReleases();
            selectUrl();
          }}
        >
          {t(locale, "settings.open_releases")}
        </Button>
      </div>
    </section>
  );
}
