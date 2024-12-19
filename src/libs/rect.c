#include <stdlib.h>
#include <stdio.h>

#include "../common.h"

struct rect_iter{
    unsigned int x_start, y_start;
    unsigned int x_end, y_end;
    struct px px; // x and y value here are the next to yield
};

void *iter_create(void *arg) {
    struct rect_iter *it = malloc(sizeof(*it));
    if (it != NULL) {
        unsigned int arg_x, arg_y, arg_w, arg_h, arg_r, arg_g, arg_b;
        if (arg == NULL) {
            return NULL;
        }
        int nmatch = sscanf(arg, "%d %d %d %d %02x%02x%02x", &arg_x, &arg_y, &arg_w, &arg_h,
                            &arg_r, &arg_g, &arg_b);
        if (nmatch != 7) {
            return NULL;
        }

        it->x_start = arg_x;
        it->x_end = arg_x + arg_w;
        it->y_start = arg_y;
        it->y_end = arg_y + arg_h;
        it->px.x = arg_x;
        it->px.y = arg_y;
        it->px.r = (unsigned char)arg_r;
        it->px.g = (unsigned char)arg_g;
        it->px.b = (unsigned char)arg_b;
    }
    return it;
}

void iter_destroy(void *it) {
    free(it);
}

static int is_done(const struct rect_iter *it) {
    return ((it->x_start == it->x_end || it->y_start == it->y_end) // empty rect
        || (it->px.y == it->y_end)); // done rect
}

static void advance(struct rect_iter *it) {
    if (it->px.x == it->x_end - 1) {
        it->px.x = it->x_start;
        it->px.y += 1;
    } else {
        it->px.x += 1;
    }
}

int iter_next(void *it_raw, struct px *px) {
    struct rect_iter *it = it_raw;
    *px = it->px;
    if (is_done(it)) {
        return 0;
    } else {
        advance(it);
        return 1;
    }
}
