# Buddy Allocator


## Core Philosophy

- dynamically divide available storage into blocks of different sizes exactly on proportion to the requirements
- instead of managing the arbitraty size, system manages block of groups in power of 2
- blocks are created on the fly by recursively splitting larger blocks in half


## System Structure and Bookkeeping

- Fixed Size Blocks: memory is managed in discrete block size, such as one, two, fourm and eight machine words
- Embedded Control Information: Each block contains a few bits of metadata indicating its size
- Availability Status: single bit in the block indicates whether it is presently free or used
- User formatting: the remain space can be formatted as user wishes
- Free Storage List: There is separate doubly connected loop for each size of free block
- System Pointers: Forward and Backward pointers


## Mechanical Work

### Spilliting

- it grabs a block from free list for the next larger size
- split this larger block into two
- one half goes to the user and other half returns to the user
- perhaps it may give back more node in the list, like is user asks 2 word from 8 word
  - 2 word return to the user and 2 and 4 word node return to the free list


### Recombination(Freeing)

The system aggressively combines the blocks 

- when the block is returned the system check for its mate(the exact block from where it is splitted from)
- to find the mate instantly the system compliments the nth order bit
- if the mate is also currently free storage, and has not been divided further, the two blocks are merged together
- recombination is also recursive


### Example-1

- A = Address of Self
- N = Size of the block

```
Mate Address=A⊕2^n
```

- block address will be: 000, 001, 010, 011, 100, 101, 110, 111
- if we divide into two halfs
- 000, 001, 010, 011 and 100, 101, 110, 111
- if we look at first bit of both blocks which is nth bit, that bit only differs for the mate and also which is size bit as well
- and that is always going to be the case, to figure out the mate ot be merge the above formula works
- The 1-Word Mates: You correctly identified that the 1-word block at 110 and the 1-word block at 111 are mates. (Bitwise, 
flipping the 0th-order bit of 110 yields 111). Because 111 is currently IN USE, the FREE block at 110 is trapped. It cannot 
recombine to form a 2-word block.
- The 2-Word Mates: The FREE 2-word block starting at 100 (which spans addresses 100 and 101) is looking for its exact 2-word 
mate. By flipping the 1st-order bit of 100, the system calculates that the mate is the 2-word block starting at 110.
- The Blocking Condition: When the allocator checks the mate at 110, it sees that the block has been subdivided into smaller 
1-word chunks. Because the mate is subdivided, the 2-word block at 100 cannot merge.
- In short, merges must happen strictly in pairs of the exact same size



### Example-2

Relative Decimal Address
	
- Binary Address
- 0, 1, 2, 3
- 0000, 0001, 0010, 0011
- 4, 5, 6, 7
- 0100, 0101, 0110, 0111
- 8, 9, 10, 11
- 1000, 1001, 1010, 1011
- 12, 13, 14, 15
- 1100, 1101, 1110, 1111


Assume we have established the following memory layout with a mix of free and allocated blocks:

- `[ Word 0 - 3 ]`  -->  4-Word Block at Address 0   (FREE)
- `[ Word 4 - 7 ]`  -->  4-Word Block at Address 4   (ALLOCATED / IN USE)
- `[ Word 8 - 9 ]`  -->  2-Word Block at Address 8   (FREE)
- `[ Word 10 - 11]` -->  2-Word Block at Address 10  (ALLOCATED / IN USE)
- `[ Word 12 - 15]` -->  4-Word Block at Address 12  (FREE)


## Refs

- Paper: A Fast Storage Allocator: Kenneth C. Knowlton
