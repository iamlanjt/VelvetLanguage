CC = gcc
CFLAGS = -Wall -Wextra -O2 -g -Iinclude

SRC = $(wildcard src/*.c)
OBJ = $(SRC:.c=.o)

all: vexlc

vexlc: $(OBJ)
	$(CC) $(CFLAGS) -o $@ $^

clean:
	del /Q src\*.o vexlc.exe 2> NUL || exit 0

