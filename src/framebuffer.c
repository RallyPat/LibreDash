#include "framebuffer.h"
#include "mmio.h"

// Mailbox property tags
#define MAILBOX_TAG_SETPHYWH 0x48003
#define MAILBOX_TAG_SETVIRTWH 0x48004
#define MAILBOX_TAG_SETVIRTOFF 0x48009
#define MAILBOX_TAG_SETDEPTH 0x48005
#define MAILBOX_TAG_SETPXLORDR 0x48006
#define MAILBOX_TAG_GETFB 0x40001
#define MAILBOX_TAG_GETPITCH 0x40008
#define MAILBOX_TAG_LAST 0

// Mailbox channels
#define MAILBOX_CH_PROP 8

// Mailbox property buffer (aligned to 16 bytes)
static volatile uint32_t __attribute__((aligned(16))) mailbox[36];

static int mailbox_call(uint32_t channel) {
    uint32_t r = ((uint32_t)((uint64_t)mailbox) & ~0xF) | (channel & 0xF);
    
    // Wait for mailbox to be ready
    while (mmio_read(MAILBOX_STATUS) & MAILBOX_FULL);
    
    // Write address to mailbox
    mmio_write(MAILBOX_WRITE, r);
    
    // Wait for response
    while (1) {
        while (mmio_read(MAILBOX_STATUS) & MAILBOX_EMPTY);
        if (r == mmio_read(MAILBOX_READ)) {
            return mailbox[1] == 0x80000000;
        }
    }
}

int fb_init(framebuffer_t *fb, uint32_t width, uint32_t height) {
    mailbox[0] = 35 * 4;
    mailbox[1] = 0;
    
    // Set physical width/height
    mailbox[2] = MAILBOX_TAG_SETPHYWH;
    mailbox[3] = 8;
    mailbox[4] = 8;
    mailbox[5] = width;
    mailbox[6] = height;
    
    // Set virtual width/height
    mailbox[7] = MAILBOX_TAG_SETVIRTWH;
    mailbox[8] = 8;
    mailbox[9] = 8;
    mailbox[10] = width;
    mailbox[11] = height;
    
    // Set virtual offset
    mailbox[12] = MAILBOX_TAG_SETVIRTOFF;
    mailbox[13] = 8;
    mailbox[14] = 8;
    mailbox[15] = 0;
    mailbox[16] = 0;
    
    // Set depth (32-bit)
    mailbox[17] = MAILBOX_TAG_SETDEPTH;
    mailbox[18] = 4;
    mailbox[19] = 4;
    mailbox[20] = 32;
    
    // Set pixel order (RGB)
    mailbox[21] = MAILBOX_TAG_SETPXLORDR;
    mailbox[22] = 4;
    mailbox[23] = 4;
    mailbox[24] = 1;
    
    // Get framebuffer
    mailbox[25] = MAILBOX_TAG_GETFB;
    mailbox[26] = 8;
    mailbox[27] = 8;
    mailbox[28] = 4096;
    mailbox[29] = 0;
    
    // Get pitch
    mailbox[30] = MAILBOX_TAG_GETPITCH;
    mailbox[31] = 4;
    mailbox[32] = 4;
    mailbox[33] = 0;
    
    // End tag
    mailbox[34] = MAILBOX_TAG_LAST;
    
    if (!mailbox_call(MAILBOX_CH_PROP)) {
        return -1;
    }
    
    fb->width = width;
    fb->height = height;
    fb->pitch = mailbox[33];
    fb->buffer = (uint32_t*)(uint64_t)(mailbox[28] & 0x3FFFFFFF);
    
    return 0;
}

void fb_clear(framebuffer_t *fb, uint32_t color) {
    for (uint32_t y = 0; y < fb->height; y++) {
        for (uint32_t x = 0; x < fb->width; x++) {
            fb->buffer[y * (fb->pitch / 4) + x] = color;
        }
    }
}

void fb_draw_pixel(framebuffer_t *fb, uint32_t x, uint32_t y, uint32_t color) {
    if (x >= fb->width || y >= fb->height) return;
    fb->buffer[y * (fb->pitch / 4) + x] = color;
}

void fb_draw_rect(framebuffer_t *fb, uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t color) {
    // Top and bottom
    for (uint32_t i = 0; i < w; i++) {
        fb_draw_pixel(fb, x + i, y, color);
        fb_draw_pixel(fb, x + i, y + h - 1, color);
    }
    // Left and right
    for (uint32_t i = 0; i < h; i++) {
        fb_draw_pixel(fb, x, y + i, color);
        fb_draw_pixel(fb, x + w - 1, y + i, color);
    }
}

void fb_draw_filled_rect(framebuffer_t *fb, uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t color) {
    for (uint32_t j = 0; j < h; j++) {
        for (uint32_t i = 0; i < w; i++) {
            fb_draw_pixel(fb, x + i, y + j, color);
        }
    }
}
