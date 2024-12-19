#ifndef PF_DYN_COMMON_H
#define PF_DYN_COMMON_H

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

#endif
