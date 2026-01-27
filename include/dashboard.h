#ifndef DASHBOARD_H
#define DASHBOARD_H

#include "framebuffer.h"

// Dashboard element types
typedef enum {
    DASH_ELEMENT_GAUGE,
    DASH_ELEMENT_LABEL,
    DASH_ELEMENT_GRAPH,
    DASH_ELEMENT_VALUE
} dash_element_type_t;

// Dashboard element structure
typedef struct {
    dash_element_type_t type;
    uint32_t x, y, width, height;
    uint32_t color;
    char label[64];
    float value;
    float min_value;
    float max_value;
} dash_element_t;

// Dashboard structure
typedef struct {
    char name[128];
    uint32_t element_count;
    dash_element_t elements[32];
} dashboard_t;

// Dashboard functions
void dashboard_init(dashboard_t *dash, const char *name);
void dashboard_add_element(dashboard_t *dash, dash_element_t *element);
void dashboard_render(dashboard_t *dash, framebuffer_t *fb);
void dashboard_update_value(dashboard_t *dash, uint32_t element_id, float value);

// .dash format parser
int dashboard_load_from_dash(dashboard_t *dash, const char *dash_data);

#endif // DASHBOARD_H
