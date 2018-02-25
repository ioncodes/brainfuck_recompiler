loc = 300
asm = [-0x06, 0x00, 0x00, 0x00]

for i in range(0, loc):
	if asm[0] == 0xff:
		if asm[1] == 0xff:
			if asm[2] == 0xff:
				asm[3] += 1
				asm[2] = 0x00
				asm[1] = 0x00
				asm[0] = 0x00
			else:
				asm[2] += 1
				asm[1] = 0x00
				asm[0] = 0x00
		else:
			asm[1] += 1
			asm[0] = 0x00
	else:
		asm[0] += 1

print(''.join('{:02x} '.format(x) for x in asm))
