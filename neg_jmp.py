loc = 549
asm = [0xfa, 0xff, 0xff, 0xff]

for i in range(0, loc):
	if asm[0] == 0x00:
		if asm[1] == 0x00:
			if asm[2] == 0x00:
				asm[3] -= 1
				asm[2] = 0xff
				asm[1] = 0xff
				asm[0] = 0xff
			else:
				asm[2] -= 1
				asm[1] = 0xff
				asm[0] = 0xff
		else:
			asm[1] -= 1
			asm[0] = 0xff
	else:
		asm[0] -= 1

print(''.join('{:02x} '.format(x) for x in asm))
