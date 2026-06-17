# CPU Scheduler


In this project, we are going to make a cpu scheduler using the MLFQ algorithm.

## MLFQ: Multi Level Feedback Queue

- schedule processes basd on the history tracking of the task
- if a task is taking more time, then mlfq automatically reduces it priority
- contains many queues
- each queue has its own time quanta, means for how much time, the task/process can be in the same queue
- once the time quanta reached, then scheduler move that task to one level down
- the lower at the level of the queue, higher the chances are to execute first
- higher the level of the queue, more the time quanta gets increased, in here we will be increasing by the factor of 2


## Simulation Implementation


### Process

- PID: unique process ID
- contains how much total time does it need to execute
- contains how many operation does it have for IO
- Priority Level
- time quanta in this level
- status: running, blocked, waiting to execute


### Process Table

- Contains all the processes as table with the ID


### Executor

- This is a single thread which executes the process
- scheduler thread will send the messages to the executor thread via a channel
- executor will have a worker thread pool which executes the task, taken from the channel
- this channel is shared b/w scheduler and executor
- executor also


### IO Executor

- read tasks from channel which is shared b/w executor and IO
- send tasks back to the mlfq to read and schedule


### Scheduler

- Contains many double ended many queues
- each queue have static time quanta and level for how much time a task can live at the same priority
- after the time quanta reached the task moved to one level down
- at each level time quanta to execute the task gets increased, means for how
  much time a task will stay at the same level gets increase
