client_2024: client_2024.c
	gcc -g -Wall -Wextra -o $@ $<

client: client.c
	gcc -Wall -Wextra -o client client.c

.PHONY: clean
clean:
	rm -f client_2024 client
