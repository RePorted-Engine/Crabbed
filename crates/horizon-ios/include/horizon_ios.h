/* Horizon's simulation core, reached from Objective-C.
 *
 * The companion to the `horizon-ios` crate, whose documentation says why these four probes exist
 * and what each of them proves. Build it with
 *   cargo rustc --release --target aarch64-apple-ios -p horizon-ios --lib --crate-type staticlib
 * and link `target/aarch64-apple-ios/release/libhorizon_ios.a`. */
#ifndef HORIZON_IOS_H
#define HORIZON_IOS_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* The smallest buffer horizon_probe() accepts, in bytes. */
#define HORIZON_PROBE_CAPACITY 4096

/* Runs Horizon's RNG, math, palette packing and NBT codec on this device, writing an
 * NUL-terminated UTF-8 report into `out`. Returns the byte count without the NUL, -1 when `out`
 * is null or `cap` is under HORIZON_PROBE_CAPACITY, and -2 when the report did not fit. */
ptrdiff_t horizon_probe(char *out, size_t cap);

/* The same value as HORIZON_PROBE_CAPACITY, for a caller that would rather ask than assume. */
size_t horizon_probe_capacity(void);

#ifdef __cplusplus
}
#endif

#endif /* HORIZON_IOS_H */
