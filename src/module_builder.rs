#[derive(Default)]
pub struct ModuleBuilder {
    types: Vec<(Vec<ValType>, Vec<ValType>)>,
    funcs: Vec<usize>,
    code_blob: Vec<u8>,
    exports: Vec<(String, u8, usize)>,
    current_func_type: usize,
    current_func_code: Vec<u8>,
    in_func: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Func(usize);

impl ValType {
    fn as_byte(&self) -> u8 {
        match self {
            ValType::I32 => 0x7f,
            ValType::I64 => 0x7e,
            ValType::F32 => 0x7d,
            ValType::F64 => 0x7c,
        }
    }
}

impl ModuleBuilder {
    pub fn into_vec(self) -> Vec<u8> {
        if self.in_func {
            panic!("Cannot turn into vector while still inside function");
        }

        // type section
        let mut type_section = vec![];
        extend_leb128_usize(&mut type_section, self.types.len());
        for (args, ret) in &self.types {
            type_section.push(0x60);
            extend_leb128_usize(&mut type_section, args.len());
            for arg in args {
                type_section.push(arg.as_byte());
            }
            extend_leb128_usize(&mut type_section, ret.len());
            for r in ret {
                type_section.push(r.as_byte());
            }
        }

        // function section
        let mut func_section = vec![];
        extend_leb128_usize(&mut func_section, self.funcs.len());
        for t in &self.funcs {
            extend_leb128_usize(&mut func_section, *t);
        }

        // export section
        let mut export_section = vec![];
        extend_leb128_usize(&mut export_section, self.exports.len());
        for (name, kind, index) in &self.exports {
            extend_leb128_usize(&mut export_section, name.len());
            export_section.extend_from_slice(name.as_bytes());
            export_section.push(*kind);
            extend_leb128_usize(&mut export_section, *index);
        }

        // Put it all together
        let mut result = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        result.push(0x01);
        extend_leb128_usize(&mut result, type_section.len());
        result.extend_from_slice(&type_section);
        result.push(0x03);
        extend_leb128_usize(&mut result, func_section.len());
        result.extend_from_slice(&func_section);
        result.push(0x07);
        extend_leb128_usize(&mut result, export_section.len());
        result.extend_from_slice(&export_section);
        result.push(0x0a);
        extend_leb128_usize(&mut result, leb_usize_len(self.funcs.len()) + self.code_blob.len());
        extend_leb128_usize(&mut result, self.funcs.len());
        result.extend_from_slice(&self.code_blob);
        result
    }

    fn typ(&mut self, args: &[ValType], ret: &[ValType]) -> usize {
        for (i,t) in self.types.iter().enumerate() {
            if (&t.0 as &[_], &t.1 as &[_]) == (args, ret) {
                return i;
            }
        }
        self.types.push((args.to_owned(), ret.to_owned()));
        self.types.len() - 1
    }
    fn emit(&mut self, bytes: &[u8]) {
        if !self.in_func {
            panic!("cannot emit code outside of a func");
        }
        self.current_func_code.extend_from_slice(bytes);
    }
    pub fn start_func(&mut self, args: &[ValType], ret: &[ValType], locals: &[ValType]) -> Func {
        if self.in_func {
            panic!("start_func cannot be called while inside a func");
        }
        self.current_func_type = self.typ(args, ret);
        self.current_func_code.clear();
        self.in_func = true;
        extend_leb128_usize(&mut self.current_func_code, locals.len());
        for local in locals {
            self.current_func_code.push(local.as_byte());
        }
        Func(self.funcs.len())
    }
    pub fn end_func(&mut self) {
        if !self.in_func {
            panic!("end_func cannot be called outside of a func");
        }
        self.emit(&[0x0b]);

        extend_leb128_usize(&mut self.code_blob, self.current_func_code.len());
        self.code_blob.extend_from_slice(&self.current_func_code);
        self.funcs.push(self.current_func_type);
        self.current_func_code.clear();
        self.in_func = false;
    }
    pub fn export_func(&mut self, f: Func, name: &str) {
        self.exports.push((name.to_owned(), 0x00, f.0));
    }
    pub fn f32_const(&mut self, x: f32) {
        self.emit(&[0x43]);
        self.emit(&x.to_le_bytes());
    }
    pub fn f32_add(&mut self) {
        self.emit(&[0x92]);
    }
    pub fn f32_sub(&mut self) {
        self.emit(&[0x93]);
    }
    pub fn f32_mul(&mut self) {
        self.emit(&[0x94]);
    }
    pub fn f32_div(&mut self) {
        self.emit(&[0x95]);
    }
}

fn extend_leb128_usize(v: &mut Vec<u8>, mut n: usize) {
    while n >= 128 {
        v.push(128 | (n & 127) as u8);
        n >>= 7;
    }
    v.push(n as u8);
}

fn leb_usize_len(mut n: usize) -> usize {
    let mut len = 1;
    while n >= 128 {
        len += 1;
    }
    len
}
