	ORG	$0 
SP: DC.L	$8000          * Stack pointer value after a reset
	DC.L	START          * Program counter value after a reset
	DC.B 1,2,3
	INCLUDE include_extra.s

	ORG	$2000		*Start at location 2000 Hex
START: MOVE.B #START+1,D3
	LINK A6,#-8
	UNLK A6
	* ORG START-$40
	MOVE.W #START,D4

