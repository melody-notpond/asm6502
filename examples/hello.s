; example from http://logicalmoon.com/2017/11/using-vs-code-to-create-a-6502-hello-world/

.define oswrch $FFEE

.org $2000		 ; code origin (like P=$2000)

_start:
	LDX #$00
letter:
	LDA message, X
	CMP #0
	BEQ finished
	JSR oswrch
	INX
	JMP letter
finished:
	RTS

message:
	.bytes "Hello, world", 13, 10, 0
