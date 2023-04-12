ORG	$0 
	DC.L	$8000          * Stack pointer value after a reset
	DC.L	START          * Program counter value after a reset

	ORG	$2000		*Start at location 2000 Hex
START: 
	BSR fn1
	MOVE #11,D0
	NOP
INCLUDE include_extra.s
	NOP