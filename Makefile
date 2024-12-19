client_2024: client_2024.c
	gcc -O -g -Wall -Wextra -o $@ $<

local: client_2024.c
	gcc -O -g -DCONNECTION_LOCAL -Wall -Wextra -o $@ $<

client-dyn: client-dyn.c
	gcc -o $@ $^ -ldl -Wl,-rpath,$(shell pwd)/libs
	
libmytest.so: mytest.c
	gcc -fPIC -shared -o $@ $<

.PHONY: clean
clean:
	rm -f client_2024 client client-dyn libmytest.so
