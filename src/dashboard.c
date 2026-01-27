#include "dashboard.h"
#include <stdint.h>

// Simple string functions for bare metal
static void str_copy(char *dest, const char *src, uint32_t max_len) {
    uint32_t i = 0;
    while (src[i] && i < max_len - 1) {
        dest[i] = src[i];
        i++;
    }
    dest[i] = '\0';
}

void dashboard_init(dashboard_t *dash, const char *name) {
    str_copy(dash->name, name, 128);
    dash->element_count = 0;
}

void dashboard_add_element(dashboard_t *dash, dash_element_t *element) {
    if (dash->element_count < 32) {
        dash->elements[dash->element_count++] = *element;
    }
}

void dashboard_update_value(dashboard_t *dash, uint32_t element_id, float value) {
    if (element_id < dash->element_count) {
        dash->elements[element_id].value = value;
    }
}

static void render_gauge(dash_element_t *elem, framebuffer_t *fb) {
    // Draw gauge background
    fb_draw_filled_rect(fb, elem->x, elem->y, elem->width, elem->height, COLOR_BLACK);
    fb_draw_rect(fb, elem->x, elem->y, elem->width, elem->height, elem->color);
    
    // Calculate fill based on value
    float percentage = (elem->value - elem->min_value) / (elem->max_value - elem->min_value);
    if (percentage < 0.0f) percentage = 0.0f;
    if (percentage > 1.0f) percentage = 1.0f;
    
    uint32_t fill_width = (uint32_t)(percentage * (elem->width - 4));
    if (fill_width > 0) {
        fb_draw_filled_rect(fb, elem->x + 2, elem->y + 2, fill_width, elem->height - 4, elem->color);
    }
}

static void render_label(dash_element_t *elem, framebuffer_t *fb) {
    // Draw label box
    fb_draw_filled_rect(fb, elem->x, elem->y, elem->width, elem->height, COLOR_BLACK);
    fb_draw_rect(fb, elem->x, elem->y, elem->width, elem->height, elem->color);
}

static void render_value(dash_element_t *elem, framebuffer_t *fb) {
    // Draw value display
    fb_draw_filled_rect(fb, elem->x, elem->y, elem->width, elem->height, COLOR_BLACK);
    fb_draw_rect(fb, elem->x, elem->y, elem->width, elem->height, elem->color);
    
    // Draw colored indicator based on value range
    uint32_t indicator_color = COLOR_GREEN;
    float percentage = (elem->value - elem->min_value) / (elem->max_value - elem->min_value);
    if (percentage > 0.8f) indicator_color = COLOR_RED;
    else if (percentage > 0.6f) indicator_color = COLOR_YELLOW;
    
    fb_draw_filled_rect(fb, elem->x + 5, elem->y + 5, 20, elem->height - 10, indicator_color);
}

static void render_graph(dash_element_t *elem, framebuffer_t *fb) {
    // Draw graph background
    fb_draw_filled_rect(fb, elem->x, elem->y, elem->width, elem->height, COLOR_BLACK);
    fb_draw_rect(fb, elem->x, elem->y, elem->width, elem->height, elem->color);
    
    // Draw grid lines
    for (uint32_t i = 1; i < 4; i++) {
        uint32_t y_pos = elem->y + (elem->height * i) / 4;
        for (uint32_t x = elem->x + 2; x < elem->x + elem->width - 2; x += 4) {
            fb_draw_pixel(fb, x, y_pos, COLOR_GRAY);
        }
    }
}

void dashboard_render(dashboard_t *dash, framebuffer_t *fb) {
    // Clear background
    fb_clear(fb, COLOR_BLACK);
    
    // Render each element
    for (uint32_t i = 0; i < dash->element_count; i++) {
        dash_element_t *elem = &dash->elements[i];
        
        switch (elem->type) {
            case DASH_ELEMENT_GAUGE:
                render_gauge(elem, fb);
                break;
            case DASH_ELEMENT_LABEL:
                render_label(elem, fb);
                break;
            case DASH_ELEMENT_VALUE:
                render_value(elem, fb);
                break;
            case DASH_ELEMENT_GRAPH:
                render_graph(elem, fb);
                break;
        }
    }
}

// Simple .dash format parser (JSON-like)
int dashboard_load_from_dash(dashboard_t *dash, const char *dash_data) {
    // This is a placeholder for .dash format parsing
    // In a full implementation, this would parse a JSON or custom format
    // For now, return success to allow manual dashboard creation
    (void)dash_data;
    return 0;
}
