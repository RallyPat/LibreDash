#ifndef FRAMEBUFFER_H
#define FRAMEBUFFER_H

#include <stdint.h>

typedef struct {
    uint32_t width;
    uint32_t height;
    uint32_t pitch;
    uint32_t *buffer;
} framebuffer_t;

// Initialize framebuffer
int fb_init(framebuffer_t *fb, uint32_t width, uint32_t height);

// Drawing functions
void fb_clear(framebuffer_t *fb, uint32_t color);
void fb_draw_pixel(framebuffer_t *fb, uint32_t x, uint32_t y, uint32_t color);
void fb_draw_rect(framebuffer_t *fb, uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t color);
void fb_draw_filled_rect(framebuffer_t *fb, uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t color);

// Color helpers
#define RGB(r, g, b) (((r) << 16) | ((g) << 8) | (b))
#define COLOR_BLACK   RGB(0, 0, 0)
#define COLOR_WHITE   RGB(255, 255, 255)
#define COLOR_RED     RGB(255, 0, 0)
#define COLOR_GREEN   RGB(0, 255, 0)
#define COLOR_BLUE    RGB(0, 0, 255)
#define COLOR_YELLOW  RGB(255, 255, 0)
#define COLOR_CYAN    RGB(0, 255, 255)
#define COLOR_MAGENTA RGB(255, 0, 255)
#define COLOR_GRAY    RGB(128, 128, 128)

#endif // FRAMEBUFFER_H
