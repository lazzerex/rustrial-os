/**
 * CPU Feature Detection Header (Assembly functions)
 */

#ifndef CPU_ASM_H
#define CPU_ASM_H

#include <stdint.h>
#include <stdbool.h>

// Assembly function prototypes
void cpu_get_vendor(char* buffer);      // Get 12-byte vendor string
uint64_t cpu_get_features(void);        // Get feature flags (EDX:ECX from leaf 1)
bool cpu_has_sse2(void);                // Check SSE2 support
bool cpu_has_avx(void);                 // Check AVX support
void cpu_get_brand(char* buffer);       // Get 48-byte brand string

#endif // CPU_ASM_H
