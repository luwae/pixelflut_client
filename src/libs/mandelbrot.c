#include <stdio.h>
#include <stdlib.h>
#include <complex.h>

#include "../common.h"

struct mandel {
    double rmin, rmax, imin, imax;
    unsigned int x, y;
};

void *iter_create(void *arg) {
    if (arg == NULL) {
        return NULL;
    }
    struct mandel *it = malloc(sizeof(*it));
    if (it == NULL) {
        return NULL;
    }
    int nmatch = sscanf(arg, "%lf %lf %lf %lf", &it->rmin, &it->rmax, &it->imin, &it->imax);
    if (nmatch != 4) {
        free(it);
        return NULL;
    }
    it->x = it->y = 0;
    return it;
}

void iter_destroy(void *it) {
    free(it);
}

static double complex mandel_iter(double complex zn, double complex c) {
    return zn * zn + c;
}

ssize_t mandel_exceeds(double complex c, size_t max_iter, double thresh) {
    double complex zn = 0 + 0 * I;
    for (size_t i = 0; i < max_iter; i++) {
        zn = mandel_iter(zn, c);
        if (creal(zn) * creal(zn) + cimag(zn) * cimag(zn) > thresh) {
            return i;
        }
    }
    return -1;
}

int iter_next(void *it_raw, struct px* px) {
    struct mandel *it = it_raw;
    double complex coord = it->rmin + (it->rmax - it->rmin)*(it->x / 512.0)
                         + (it->imin + (it->imax - it->imin)*((512-(it->y + 1)) / 512.0)) * I;
    ssize_t exceeds = mandel_exceeds(coord, 20, 100000.0);
    if (exceeds == -1) {
        px->r = px->g = px->b = 255;
    } else {
        px->r = px->g = px->b = 10*exceeds;
    }
    px->x = it->x;
    px->y = it->y;
    
    // advance x and y
    it->x += 1;
    if (it->x == 512) {
        it->x = 0;
        it->y += 1;
        if (it->y == 512) {
            it->y = 0;
        }
    }
    return 1;
}
