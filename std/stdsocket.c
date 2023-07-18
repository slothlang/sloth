#include "stdlib.c"
#include <netinet/in.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

int serversock(int PORT, char *addr, int backlog) {
  int opt = 1;
  int sock, new_sock;
  struct sockaddr_in address;
  int addrlen = sizeof(address);

  if ((sock = socket(PF_INET, SOCK_STREAM, 0)) < 0) {
    perror("socket failed");
    exit(EXIT_FAILURE);
  }

  if (setsockopt(sock, SOL_SOCKET, SO_REUSEADDR | SO_REUSEPORT, &opt,
                 sizeof(opt)) < 0) {
    perror("setsockopt");
    exit(EXIT_FAILURE);
  }

  address.sin_family = AF_INET;
  if (sequals(addr, "auto")) {
    address.sin_addr.s_addr = INADDR_ANY;
  } else {
    inet_aton(addr, &address.sin_addr.s_addr);
  }
  address.sin_port = htons(PORT);

  if (bind(sock, (struct sockaddr *)&address, sizeof(address)) < 0) {
    perror("bind");
    exit(EXIT_FAILURE);
  }
  if (listen(sock, backlog) < 0) {
    perror("listen");
    exit(EXIT_FAILURE);
  }

  if ((new_sock = accept(sock, (struct sockaddr *)&address,
                         (socklen_t *)&addrlen)) < 0) {
    perror("accept");
    exit(EXIT_FAILURE);
  }
  return new_sock;
}

int clientsock(int PORT, char *server_ip) {
  struct sockaddr_in serv_addr;
  int sock = socket(PF_INET, SOCK_STREAM, 0);
  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(PORT);

  inet_pton(AF_INET, server_ip, &serv_addr.sin_addr);
  int status = connect(sock, (struct sockaddr *)&serv_addr, sizeof(serv_addr));
  return sock;
}

char *recvsock(int soc) {
  char *buf = malloc(1024);
  int valread = read(soc, buf, 1024);
  return buf;
}

void sendsock(char *msg, int soc) { send(soc, msg, strlen(msg), 0); }

void closesock(int soc, bool server) {
  close(soc);
  if (server) {
    shutdown(soc, SHUT_RDWR);
  }
}
