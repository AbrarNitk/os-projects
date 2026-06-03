# Pipong: Two Proesses Ping Pong With Pipes


In this project there are going to be process, they do IPC with pipes.


## `pipe`


- create a uni and unidirectional pipes so that processes can do the IPC.
- with first descriptor we can read the data
- with second descriptor we can write the data

```
int pipe(int pipefd[2]);
```

## Mechanics

- terminal runs a parent process
- parent process create two pipes
  - p1(r1, w1)
  - p2(r2, w2)
- parent fork the process and create a child using the fork command
- in p1-w1 parent writes it's message
- in p1-r1 child reads the parent message
- in p2-w2 child write the message
- in p2-r2 parent reads the child message


### refs

- [man page](https://man7.org/linux/man-pages/man2/pipe.2.html)
