// Information about the machine and build, for rows of measurement results pasted into CSV.

/** Name of the computer, set when building: `PC_NAME="desktop" npm run build` */
export const PC_NAME = __PC_NAME__;

export const BUILD = import.meta.env.PROD ? "release" : "debug";

/** Local date and time, `YYYY-MM-DD HH:MM:SS` */
export function date() {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, "0");
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
    `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  );
}

/** `[name, version]` of the browser, parsed from the user agent */
export function browser() {
  const userAgent = navigator.userAgent;
  // order matters, Edge and Opera also contain "Chrome/", Chrome also contains "Safari/"
  for (const [token, name] of [
    ["Firefox/", "Firefox"],
    ["Edg/", "Edge"],
    ["OPR/", "Opera"],
    ["Chrome/", "Chrome"],
    ["Version/", "Safari"],
  ]) {
    const index = userAgent.indexOf(token);
    if (index >= 0) {
      const version = userAgent.slice(index + token.length).split(/[ ;)]/)[0];
      return [name, version];
    }
  }
  return [userAgent, ""];
}

/** Operating system as reported by the browser */
export function os() {
  return navigator.platform;
}

/** Logical CPU cores */
export function cpuCores() {
  return navigator.hardwareConcurrency ?? 0;
}

/** Joins the fields into one CSV row, quoting the ones that need it */
export function csvRow(fields) {
  return fields
    .map((field) => {
      const text = String(field);
      return /[,"\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
    })
    .join(",");
}
