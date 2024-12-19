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
#include <sys/mman.h>
#include <fcntl.h>
#include <dlfcn.h>

#ifdef CONNECTION_LOCAL
#define ADDRESS "127.0.0.1"
#define PORT 1337
#define WRITE_PIXEL write_pixel
#else
#define ADDRESS "193.196.38.206"
#define PORT 1234
#define WRITE_PIXEL write_batched
#endif

static unsigned long long pixel_count = 0;

#define LOADED_FILE_SIZE (256*1024ull)
void *loaded_file;


#define PANIC(msg) do { fprintf(stderr, "%s\n", msg); exit(1); } while (0)
#define ASSERT(cond, msg) do { if (!(cond)) PANIC(msg); } while (0)
#define EPANIC(msg) do { perror(msg); exit(1); } while (0)
#define EASSERT(cond, msg) do { if (!(cond)) EPANIC(msg); } while (0)

struct px {
    unsigned int x;
    unsigned int y;
    unsigned char r;
    unsigned char g;
    unsigned char b;
};

typedef void *(*iter_create_t)(void);
typedef void (*iter_destroy_t)(void *);
typedef int (*iter_next_t)(void *, struct px *);

struct barnsley_iter {
    double x, y;
};

void barnsley_iter_init(struct barnsley_iter *it) {
    memset(it, 0, sizeof(*it));
}

int barnsley_iter(void *barn, struct px *px) {
    struct barnsley_iter *it = barn;
    double xn, yn;
    double num = rand() / (double)RAND_MAX;
    if (num < 0.01) {
        xn = 0.0;
        yn = 0.16 * it->y;
    } else if (num < 0.86) {
        xn = 0.85 * it->x + 0.04 * it->y;
        yn = -0.04 * it->x + 0.85 * it->y + 1.6;
    } else if (num < 0.93) {
        xn = 0.2 * it->x - 0.26 * it->y;
        yn = 0.23 * it->x + 0.22 * it->y + 1.6;
    } else {
        xn = -0.15 * it->x + 0.28 * it->y;
        yn = 0.26 * it->x + 0.24 * it->y + 0.44;
    }
    it->x = xn;
    it->y = yn;

    px->x = (unsigned int)((it->x + 5.0) * 50.0);
    px->y = (unsigned int)(it->y * 50.0);
    px->r = 250 - (px->y >> 3);
    px->g = 50 + (px->y >> 2);
    px->b = 50 - (px->y >> 4);
    return 1;
}

struct file_iter {
    unsigned int x, y;
};

void file_iter_init(struct file_iter *it) {
    memset(it, 0, sizeof(*it));
}

int file_iter(void *fit, struct px *px) {
    struct file_iter *it = fit;
    px->x = it->x;
    px->y = it->y;
    unsigned char col = ((char*)loaded_file)[it->x + 512 * it->y];
    px->r = px->g = px->b = col;
    // advance
    it->x += 1;
    if (it->x == 512) {
        it->x = 0;
        it->y += 1;
        if (it->y == 512) {
            // start at the front again
            it->y = 0;
        }
    }
    return 1;
}

int rand_iter(void *_unused, struct px *px) {
    (void) _unused;
    int pos = rand();
    px->x = pos & 0x1ff;
    px->y = (pos >> 16) & 0x1ff;
    int color = rand();
    px->r = color & 0xff;
    px->g = 0;
    px->b = 0;
    px->g = (color >> 8) & 0xff;
    px->b = (color >> 16) & 0xff;

    // usleep(1000);
    return 1;
}

struct square_iter {
    unsigned int x_start, y_start;
    unsigned int x_end, y_end;
    struct px px; // x and y value here are the next to yield
};

void square_iter_init(struct square_iter *it, unsigned int x_start, unsigned int y_start,
                      unsigned int x_end, unsigned int y_end,
                      unsigned char r, unsigned char g, unsigned char b)
{
    ASSERT(x_start <= x_end && y_start <= y_end, "negative-dimension square");
    it->x_start = x_start;
    it->y_start = y_start;
    it->x_end = x_end;
    it->y_end = y_end;
    it->px.x = x_start;
    it->px.y = y_start;
    it->px.r = r;
    it->px.g = g;
    it->px.b = b;
}

int square_iter_done(struct square_iter *it) {
    return ((it->x_start == it->x_end || it->y_start == it->y_end) // empty square
        || (it->px.y == it->y_end)); // done square
}

void square_iter_advance(struct square_iter *it) {
    if (it->px.x == it->x_end - 1) {
        it->px.x = it->x_start;
        it->px.y += 1;
    } else {
        it->px.x += 1;
    }
}

int square_iter(void *iter_void, struct px *px) {
    struct square_iter *it = iter_void;
    *px = it->px;
    if (square_iter_done(it)) {
        return 0;
    } else {
        square_iter_advance(it);
        return 1;
    }
}

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

    // other funny setup

    int file_fd = open("fs.img", O_RDONLY);
    EASSERT(file_fd != -1, "open");
    loaded_file = mmap(NULL, LOADED_FILE_SIZE, PROT_READ, MAP_PRIVATE, file_fd, 0);
    EASSERT(loaded_file != MAP_FAILED, "mmap");
    
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

    /*
    struct square_iter square;
    square_iter_init(&square, 0, 0, 512, 512, 255, 255, 255);
    drain_iter(sock, &square, square_iter);

    struct barnsley_iter barn;
    barnsley_iter_init(&barn);
    drain_iter(sock, &barn, barnsley_iter);
    
    struct file_iter fit;
    file_iter_init(&fit);
    drain_iter(sock, &fit, file_iter);
    */

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

    void *it = iter_create();
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
