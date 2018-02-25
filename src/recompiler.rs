use token::Token;

pub struct Recompiler {
    pub bytes: Vec<u8>,
    indices: Vec<usize>,
}

impl Recompiler {
    pub fn new() -> Recompiler {
        Recompiler {
            bytes: Vec::<u8>::new(),
            indices: Vec::<usize>::new(),
        }
    }

    pub fn translate(&mut self, bf: String) {
        let tokens = self.tokenize(bf);
        for token in tokens {
            match token.symbol {
                '>' => {
                    let mut asm = sam!(x64 => "add rax, 0");
                    let i = asm.len() - 1;
                    asm[i] = token.amount as u8;
                    self.bytes.extend(asm);
                },
                '<' => {
                    let mut asm = sam!(x64 => "sub rax, 0");
                    let i = asm.len() - 1;
                    asm[i] = token.amount as u8;
                    self.bytes.extend(asm);
                },
                '+' => {
                    let mut asm = sam!(x64 => "add byte ptr [rax], 0");
                    let i = asm.len() - 1;
                    asm[i] = token.amount as u8;
                    self.bytes.extend(asm);
                },
                '-' => {
                    let mut asm = sam!(x64 => "sub byte ptr [rax], 0");
                    let i = asm.len() - 1;
                    asm[i] = token.amount as u8;
                    self.bytes.extend(asm);
                },
                '.' => self.bytes.extend(sam!(x64 => "
                    push rax
                    mov rdi, qword ptr [rax]
                    call r12
                    pop rax
                ")), // we need to push rax, or print will destroy it
                ',' => self.bytes.extend(sam!(x64 => "nop")),
                '[' => {
                    self.bytes.extend(sam!(x64 => "nop"));
                    self.bytes.extend(sam!(x64 => "nop"));
                    self.bytes.extend(sam!(x64 => "nop"));
                    self.bytes.extend(sam!(x64 => "nop"));
                    self.bytes.extend(sam!(x64 => "nop"));
                    self.bytes.extend(sam!(x64 => "nop"));
                    self.indices.push(self.bytes.len() - 1);
                },
                ']' => {
                    let index = self.indices.pop().unwrap();
                    let mut length = self.bytes.len();
                    let mut size = length - index;
                    if size > 129 {
                        let bytes = self.calc_pos_jmp(size + 1);
                        self.bytes[index] = bytes[3];
                        self.bytes[index - 1] = bytes[2];
                        self.bytes[index - 2] = bytes[1];
                        self.bytes[index - 3] = bytes[0];
                        self.bytes[index - 4] = 0x84;
                        self.bytes[index - 5] = 0x0f;
                    } else {
                        self.bytes[index - 5] = 0x74;
                        self.bytes[index - 4] = (size + 12) as u8; // + 12 bytes for after [
                        /*self.bytes.remove(index - 3);
                        self.bytes.remove(index - 2);
                        self.bytes.remove(index - 1);
                        self.bytes.remove(index);*/
                    }
                    let long = sam!(x64 => "
                        cmp byte ptr [rax], 0
                        jne -300
                    ");
                    let short = sam!(x64 => "
                        cmp byte ptr [rax], 0
                        jne -0
                    ");
                    if length + short.len() > 126 {
                        /*
                            0f 85 xx xx xx xx
                        */
                        self.bytes.extend(long);
                        length = self.bytes.len();
                        size = length - index - 3; // - 3 bytes for after ]
                        let bytes = self.calc_neg_jmp(size);
                        self.bytes[length - 4] = bytes[0];
                        self.bytes[length - 3] = bytes[1];
                        self.bytes[length - 2] = bytes[2];
                        self.bytes[length - 1] = bytes[3];
                    } else {
                        /*
                            75 xx
                        */
                        self.bytes.extend(short);
                        self.bytes[length + 4] = (0xfe - size - 6) as u8;
                        self.bytes.extend(sam!(x64 => "nop"));
                        self.bytes.extend(sam!(x64 => "nop"));
                        self.bytes.extend(sam!(x64 => "nop"));
                        self.bytes.extend(sam!(x64 => "nop"));
                    }
                },
                _ => (),
            }
        }
    }

    fn tokenize(&self, bf: String) -> Vec<Token> {
        let mut last_token = bf.chars().nth(0).unwrap().to_string();
        let mut last_token_counter = 0;
        let mut tokens = Vec::<Token>::new();
        for instruction in bf.chars() {
            match instruction {
                '+' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from("+")),
                '-' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from("-")),
                '<' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from("<")),
                '>' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from(">")),
                '[' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from("[")),
                ']' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from("]")),
                '.' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from(".")),
                ',' => self.check(&mut last_token, &mut last_token_counter, &mut tokens, &mut String::from(",")),
                _ => (), // ignore comments, etc
            };
        }
        tokens
    }

    fn check(&self, last_token: &mut String, last_token_counter: &mut usize, tokens: &mut Vec<Token>, l_token: &mut String) {
        if l_token == "[" || l_token == "]" || l_token == "." || l_token == "," {
            tokens.push(Token {
                symbol: last_token.chars().nth(0).unwrap(),
                amount: *last_token_counter,
            });
            *last_token = l_token.clone();
            *last_token_counter = 1;
        } else {
            if last_token == l_token {
                *last_token_counter += 1;
            } else {
                tokens.push(Token {
                    symbol: last_token.chars().nth(0).unwrap(),
                    amount: *last_token_counter,
                });
                *last_token = l_token.clone();
                *last_token_counter = 1;
            }
        }
    }

    fn calc_neg_jmp(&self, size: usize) -> [u8; 4] {
        let mut asm = [0xfa, 0xff, 0xff, 0xff];

        for _ in 0..size {
            if asm[0] == 0x00 {
                if asm[1] == 0x00 {
                    if asm[2] == 0x00 {
                        asm[3] -= 1;
                        asm[2] = 0xff;
                        asm[1] = 0xff;
                        asm[0] = 0xff;
                    } else {
                        asm[2] -= 1;
                        asm[1] = 0xff;
                        asm[0] = 0xff;
                    }
                } else {
                    asm[1] -= 1;
                    asm[0] = 0xff;
                }
            } else {
                asm[0] -= 1;
            }
        }

        asm
    }

    fn calc_pos_jmp(&self, size: usize) -> [u8; 4] {
        let mut asm: [isize; 4] = [-0x06, 0x00, 0x00, 0x00];

        for _ in 0..size {
            if asm[0] == 0xff {
                if asm[1] == 0xff {
                    if asm[2] == 0xff {
                        asm[3] += 1;
                        asm[2] = 0x00;
                        asm[1] = 0x00;
                        asm[0] = 0x00;
                    } else {
                        asm[2] += 1;
                        asm[1] = 0x00;
                        asm[0] = 0x00;
                    }
                } else {
                    asm[1] += 1;
                    asm[0] = 0x00;
                }
            } else {
                asm[0] += 1;
            }
        }

        [asm[0] as u8, asm[1] as u8, asm[2] as u8, asm[3] as u8]
    }
}