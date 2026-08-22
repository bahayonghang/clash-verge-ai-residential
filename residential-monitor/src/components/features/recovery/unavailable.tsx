import { t, type UiLocale } from "../../../i18n";
import { formatTemplate } from "../../../lib/utils";
import { Card, CardDescription, CardHeader, CardTitle } from "../../ui/card";

export function UnavailablePage({
  locale,
  name,
  until
}: {
  locale: UiLocale;
  name: string;
  until: string;
}) {
  return (
    <Card className="max-w-xl">
      <CardHeader>
        <CardTitle>{name}</CardTitle>
        <CardDescription>{formatTemplate(t(locale, "unavailable.body"), { until })}</CardDescription>
      </CardHeader>
    </Card>
  );
}
