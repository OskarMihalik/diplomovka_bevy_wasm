// The machine the benchmarks run on: keeping the screen awake, noticing a locked screen, and what
// goes into meta.json. KDE Plasma on Wayland, see PLAN.md section 4.5.

import { execFile, execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";

/** stdout of a command, "" if it fails or doesn't exist */
export function sh(file, ...args) {
  try {
    return execFileSync(file, args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  } catch {
    return "";
  }
}

/** true on AC, false on battery, null if there is no power supply info */
export function onAcPower() {
  const dir = "/sys/class/power_supply";
  if (!fs.existsSync(dir)) return null;
  for (const supply of fs.readdirSync(dir)) {
    const read = (name) => {
      try {
        return fs.readFileSync(`${dir}/${supply}/${name}`, "utf8").trim();
      } catch {
        return "";
      }
    };
    if (read("type") === "Mains") return read("online") === "1";
  }
  return null;
}

// apps that draw on the GPU or keep the CPU busy while they are open
const BUSY_APPS = /^(discord|obsidian|code|slack|spotify|steam|chrome|chromium|firefox|thunderbird|teams|telegram|signal)/i;

/** Running apps that compete with the benchmark, by process name */
export function busyApps() {
  const names = sh("ps", "-u", String(os.userInfo().uid), "-o", "comm=").split("\n");
  // crash reporters belong to an app that is listed anyway
  return [
    ...new Set(names.map((name) => name.trim()).filter((name) => BUSY_APPS.test(name) && !/crashpad/.test(name))),
  ];
}

/** The `count` processes using the most CPU right now, `name cpu%` */
export function topProcesses(count = 8) {
  return sh("ps", "-eo", "pcpu=,comm=", "--sort=-pcpu")
    .split("\n")
    .slice(0, count)
    .map((line) => line.trim().replace(/^(\S+)\s+(.*)$/, "$2 $1%"));
}

/**
 * Stops KDE from dimming, turning off or locking the screen: on Wayland, Firefox draws nothing
 * while the screen is locked. Returns a function that ends it.
 */
export function keepScreenAwake() {
  const inhibit = sh("which", "kde-inhibit")
    ? spawn("kde-inhibit", ["--power", "--screenSaver", "sleep", "infinity"], { stdio: "ignore" })
    : null;
  inhibit?.on("error", () => {});
  // kde-inhibit alone wasn't enough to be sure, this resets the idle timers
  const timer = setInterval(() => {
    execFile(
      "qdbus6",
      ["org.freedesktop.ScreenSaver", "/ScreenSaver", "org.freedesktop.ScreenSaver.SimulateUserActivity"],
      () => {},
    );
  }, 30_000);
  return () => {
    clearInterval(timer);
    inhibit?.kill();
  };
}

/** true if the session's screen is locked, false if not or unknown */
export function screenLocked() {
  const session = process.env.XDG_SESSION_ID;
  if (!session) return false;
  return sh("loginctl", "show-session", session, "-p", "LockedHint", "--value") === "yes";
}

/** Everything about the machine and the code that belongs next to the results */
export function machineInfo(root) {
  const git = (...args) => sh("git", "-C", root, ...args);
  return {
    hostname: os.hostname(),
    git_commit: git("rev-parse", "HEAD"),
    git_dirty: git("status", "--porcelain", "--untracked-files=no") !== "",
    kernel: os.release(),
    os: sh("sh", "-c", ". /etc/os-release && echo $PRETTY_NAME"),
    cpu: os.cpus()[0]?.model ?? "",
    cpu_cores: os.cpus().length,
    memory_gib: Math.round(os.totalmem() / 2 ** 30),
    // one line per architecture (amd64, i386)
    mesa: sh("dpkg-query", "-W", "-f=${Version}\n", "libgl1-mesa-dri").split("\n")[0],
    power_profile: sh("powerprofilesctl", "get"),
    ac_power: onAcPower(),
    session_type: process.env.XDG_SESSION_TYPE ?? "",
    top_processes: topProcesses(),
  };
}
