// loupe.h — C API header for the loupe library
// Reference only — the actual library is built from Zig source.

#ifndef LOUPE_H
#define LOUPE_H

#include <stdint.h>

typedef struct {
    double x, y, width, height, confidence;
} LoupeFaceResult;

typedef struct {
    const char *text;
    double x, y, width, height, confidence;
} LoupeOcrResult;

// Image lifecycle
void *loupe_load_image(const char *path);
void loupe_free_image(void *handle);
int loupe_save_image(void *handle, const char *path);

// Face detection
int loupe_detect_faces(void *handle, LoupeFaceResult **out_faces, uint32_t *out_count);
void *loupe_blur_faces(void *handle, const LoupeFaceResult *faces, uint32_t count, int mode);

// OCR
int loupe_recognize_text(void *handle, LoupeOcrResult **out_results, uint32_t *out_count);
void loupe_free_ocr_results(LoupeOcrResult *results, uint32_t count);

// Memory
void loupe_free(void *ptr);

#endif
