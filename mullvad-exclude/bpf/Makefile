arch != uname -m
outputs  = $(patsubst %.c, %.$(arch), $(wildcard *.c))

.PHONY: all clean

all: $(outputs)

clean:
	rm -f $(outputs)

%.$(arch): %.c
	@echo hello $(arch)
	clang $< -target bpf -g -c -o $@
