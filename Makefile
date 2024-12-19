IN_DIR := src
OUT_DIR := build

LIBS_SRCS := $(wildcard $(IN_DIR)/libs/*.c)
LIBS_BASE := $(basename $(notdir $(LIBS_SRCS)))
LIBS := $(patsubst %,$(OUT_DIR)/libs/lib%.so,$(LIBS_BASE))
all: $(OUT_DIR)/client-dyn $(LIBS)

$(OUT_DIR)/client_2024: $(IN_DIR)/client_2024.c $(OUT_DIR)
	gcc -O -g -Wall -Wextra -o $@ $<

$(OUT_DIR)/local: $(IN_DIR)/client_2024.c $(OUT_DIR)
	gcc -O -g -DCONNECTION_LOCAL -Wall -Wextra -o $@ $<

$(OUT_DIR)/client-dyn: $(IN_DIR)/client-dyn.c $(OUT_DIR)
	gcc -o $@ $< -ldl -Wl,-rpath,$(shell pwd)/$(OUT_DIR)/libs
	
$(OUT_DIR)/libs/lib%.so: $(IN_DIR)/libs/%.c $(OUT_DIR) $(OUT_DIR)/libs
	gcc -fPIC -shared -o $@ $<

$(OUT_DIR):
	mkdir $@
$(OUT_DIR)/libs:
	mkdir $@

.PHONY: clean
clean:
	rm -fr $(OUT_DIR)
