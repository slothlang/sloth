#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>
#define PORT 8080

int serversock() {
	int opt = 1;
	int sock = socket(AF_INET, SOCK_STREAM, 0);
	struct sockaddr_in address;
	int addrlen = sizeof(address);
	setsockopt(sock, SOL_SOCKET, SO_REUSEADDR | SO_REUSEPORT, &opt, sizeof(opt));
	address.sin_family = AF_INET;
	address.sin_addr.s_addr = INADDR_ANY;
	address.sin_port = htons(PORT);

	bind(sock, (struct sockaddr*)&address, sizeof(address));
	listen(sock, 3);
	int new_sock = accept(sock, (struct sockaddr*)&address, (socklen_t*)&addrlen);
	return new_sock;
}

int clientsock() {
	struct sockaddr_in serv_addr;
	int sock = socket(AF_INET, SOCK_STREAM, 0);
	serv_addr.sin_family = AF_INET;
	serv_addr.sin_port = htons(PORT);
	
	inet_pton(AF_INET, "127.0.0.1", &serv_addr.sin_addr);
	int status = connect(sock, (struct sockaddr*)&serv_addr, sizeof(serv_addr));
	return sock;
}

char* recvsock(int soc) {
	char* buf = malloc(1024);
	int valread = read(soc, buf, 1024);
	return buf;
}

void sendsock(char* msg, int soc) {
	send(soc, msg, strlen(msg), 0);
}

void closesock(int soc) {
	close(soc);
}
