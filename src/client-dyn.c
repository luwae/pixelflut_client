#include <sys/socket.h>
#include <stdio.h>
#include <stdlib.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <time.h>
#include <stdlib.h>
#include <dlfcn.h>

#include "common.h"

#ifdef CONNECTION_LOCAL
#define ADDRESS "127.0.0.1"
#define PORT 1337
#define WRITE_PIXEL write_pixel_bin
#else
#define ADDRESS "193.196.38.206"
#define PORT 1234
#define WRITE_PIXEL write_batched
#endif

static unsigned long long pixel_count = 0;

typedef void *(*iter_create_t)(void *);
typedef void (*iter_destroy_t)(void *);
typedef int (*iter_next_t)(void *, struct px *);

#define BATCH_SIZE 1024
// strlen("PX xxxx xxxx rrggbbaa\n")
#define MAX_LEN 22
char batch_buf[BATCH_SIZE];
size_t batch_bufp = 0;

void write_batched_flush_once(int fd) {
    ssize_t written = write(fd, batch_buf, batch_bufp);
    EASSERT(written != -1, "write");
    ASSERT(written != 0, "written 0 bytes?");
    // move
    memmove(batch_buf, batch_buf + written, batch_bufp - written);
    batch_bufp -= written;
}

void write_batched(int fd, const struct px *p) {
    while (batch_bufp + MAX_LEN >= BATCH_SIZE) {
        write_batched_flush_once(fd);
    }

    ssize_t len = sprintf(batch_buf + batch_bufp, "PX %d %d %02x%02x%02x\n", p->x, p->y, p->r, p->g, p->b);
    ASSERT(len > 0, "sprintf");
    batch_bufp += (size_t)len;
    pixel_count += 1;
}

void write_batched_flush(int fd) {
    while (batch_bufp > 0) {
        write_batched_flush_once(fd);
    }
}

void write_pixel(int fd, const struct px *p) {
    static char buf[100];
    ssize_t len = sprintf(buf, "PX %d %d %02x%02x%02x\n", p->x, p->y, p->r, p->g, p->b);
    ASSERT(len > 0, "sprintf");

#ifdef DEBUG_OUTPUT
    printf(buf); // test output
#endif

    ssize_t written = write(fd, buf, len);
    EASSERT(written != -1, "write");
    ASSERT(written == len, "write hasn't written all bytes");
    pixel_count += 1;
}

void write_pixel_bin(int fd, const struct px *px) {
    static char buf[8];
    buf[0] = 'P';
    buf[1] = px->x & 0xff;
    buf[2] = (px->x >> 8) & 0xff;
    buf[3] = px->y & 0xff;
    buf[4] = (px->y >> 8) & 0xff;
    buf[5] = px->r;
    buf[6] = px->g;
    buf[7] = px->b;
    ssize_t written = write(fd, buf, 8);
    EASSERT(written != -1, "write");
    ASSERT(written == 8, "write hasn't written all bytes");
    pixel_count += 1;
}

void drain_iter(int fd, void *iter, iter_next_t func) {
    struct px px;
    while (func(iter, &px)) {
        WRITE_PIXEL(fd, &px);
    }
}

void sig_handler(int s) {
    (void) s;
    printf("\n%llu pixels written\n", pixel_count);
    exit(1);
}

int main(int argc, char *argv[]) {
    srand(time(NULL));

    struct sigaction action;
    action.sa_handler = sig_handler;
    sigemptyset(&action.sa_mask);
    action.sa_flags = 0;
    sigaction(SIGINT, &action, NULL);
    
    // real program start    

    int sock = socket(AF_INET, SOCK_STREAM, 0);
    EASSERT(sock != -1, "socket");

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(PORT),
        .sin_addr = { .s_addr = inet_addr(ADDRESS) }
    };

    int status = connect(sock, (struct sockaddr *)&addr, sizeof(addr));
    EASSERT(status != -1, "connect");

    // dl start
    
    const char *err;

    ASSERT(argc > 1, "missing command line argument");
    
    void *obj = dlopen(argv[1], RTLD_NOW);
    err = dlerror();
    ASSERT(err == NULL, err);

    iter_create_t iter_create = dlsym(obj, "iter_create");
    err = dlerror();
    ASSERT(err == NULL, err);
    iter_destroy_t iter_destroy = dlsym(obj, "iter_destroy");
    err = dlerror();
    ASSERT(err == NULL, err);
    iter_next_t iter_next = dlsym(obj, "iter_next");
    err = dlerror();
    ASSERT(err == NULL, err);

    void *it = iter_create(argv[2]);
    ASSERT(it != NULL, "iter_create returned NULL");
    struct px px;
    while (iter_next(it, &px)) {
        write_batched(sock, &px);
    }
    write_batched_flush(sock);
    iter_destroy(it);

    dlclose(obj);
    
    close(sock);

    printf("\n%llu pixels written\n", pixel_count);
}
