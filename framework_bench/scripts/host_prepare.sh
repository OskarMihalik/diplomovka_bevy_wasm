#!/usr/bin/env bash
# Puts the host into a stable state for a benchmark batch: turbo off and the performance governor
# on every CPU. With turbo on, the pilot reached 98-100 °C and throttled, so clock speeds varied
# between cells. Needs root.
#
#   sudo framework_bench/scripts/host_prepare.sh            # before a batch
#   sudo framework_bench/scripts/host_prepare.sh --restore  # afterwards: turbo on, powersave
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "run with sudo" >&2
  exit 1
fi

case ${1:-} in
  "") no_turbo=1 governor=performance profile=performance ;;
  --restore) no_turbo=0 governor=powersave profile=balanced ;;
  *) echo "usage: $0 [--restore]" >&2; exit 2 ;;
esac

echo "$no_turbo" > /sys/devices/system/cpu/intel_pstate/no_turbo
for path in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
  echo "$governor" > "$path"
done
if command -v powerprofilesctl > /dev/null; then
  powerprofilesctl set "$profile" || true
fi

echo "no_turbo:  $(cat /sys/devices/system/cpu/intel_pstate/no_turbo)"
echo "governors: $(sort -u /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor | tr '\n' ' ')"
if command -v powerprofilesctl > /dev/null; then
  echo "profile:   $(powerprofilesctl get)"
fi

on_ac=unknown
for supply in /sys/class/power_supply/*; do
  if [[ $(cat "$supply/type" 2> /dev/null) == Mains ]]; then
    on_ac=$([[ $(cat "$supply/online") == 1 ]] && echo yes || echo no)
  fi
done
echo "on AC:     $on_ac"
if [[ $on_ac == no ]]; then
  echo "warning: running on battery; plug in the charger before a batch" >&2
fi

if [[ -z ${1:-} ]]; then
  echo
  echo "Also set the Legion power mode to Performance (Fn+Q) and close other applications."
fi
