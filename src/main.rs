use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::ops::Index;

pub struct JavaBytecodeReader {
    offset: usize,
    lenght: usize,
    raw_data: Vec<u8>,
}

impl JavaBytecodeReader {
    pub fn new(raw_data: Vec<u8>) -> JavaBytecodeReader {
        JavaBytecodeReader {
            offset: 0,
            lenght: raw_data.len(),
            raw_data,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0
    }

    pub fn has_next(&self) -> bool {
        self.offset < self.lenght
    }

    pub fn read_as_u64(&mut self, step: usize) -> u64 {
        if step < 1 || step > 4 {
            return 0_u64;
        }
        let mut end: usize = self.offset + step;
        if end > self.lenght {
            panic!("JavaBytecodeReader: incorrect length.")
        }

        let mut result = self.raw_data[self.offset..end].to_vec();
        result.reverse();
        result.resize(8, 0u8);

        self.offset = end;
        u64::from_le_bytes(
            result
                .as_slice()
                .try_into()
                .expect("JavaBytecodeReader: incorrect length."),
        )
    }

    pub fn b1(&mut self) -> u8 {
        self.read_as_u64(1) as u8
    }

    pub fn b2(&mut self) -> u16 {
        self.read_as_u64(2) as u16
    }

    pub fn b4(&mut self) -> u32 {
        self.read_as_u64(4) as u32
    }

    pub fn read_as_string(&mut self, lenght: usize) -> String {
        let mut end: usize = self.offset + lenght;
        if end > self.lenght {
            panic!("JavaBytecodeReader: incorrect length.")
        }
        let result = self.raw_data[self.offset..end].to_vec();

        self.offset = end;
        String::from_utf8(result).unwrap()
    }
    pub fn read_as_vec(&mut self, lenght: usize) -> Vec<u8> {
        let mut end: usize = self.offset + lenght;
        if end > self.lenght {
            panic!("JavaBytecodeReader: incorrect length.")
        }
        let info = self.raw_data[self.offset..end].to_vec();
        self.offset = end;
        info
    }
}

#[derive(Debug)]
enum Constant {
    Utf8(ConstantUtf8),
    Class(ConstantClass),
    String(ConstantString),
    Fieldref(ConstantFieldref),
    Methodref(ConstantMethodref),
    NameAndType(ConstantNameAndType),
}

#[derive(Debug)]
pub struct ConstantUtf8 {
    tag: u8,
    data: String,
}

#[derive(Debug)]
pub struct ConstantClass {
    tag: u8,
    name_index: u16,
}

#[derive(Debug)]
pub struct ConstantString {
    tag: u8,
    string_index: u16,
}

#[derive(Debug)]
pub struct ConstantFieldref {
    tag: u8,
    class_index: u16,
    name_and_type_index: u16,
}

#[derive(Debug)]
pub struct ConstantMethodref {
    tag: u8,
    class_index: u16,
    name_and_type_index: u16,
}

#[derive(Debug)]
pub struct ConstantNameAndType {
    tag: u8,
    name_index: u16,
    descriptor_index: u16,
}
/**********************/
#[derive(Default, Debug)]
pub struct Field {
    access_flags: u16,
    name: String,
    descriptor: String,
    attributes: Vec<Attributes>,
}

#[derive(Default, Debug)]
pub struct Method {
    access_flags: u16,
    name: String,
    descriptor: String,
    attributes: Vec<Attributes>,
}

#[derive(Default, Debug)]
pub struct Attributes {
    attribute_name: String,
    info: Vec<u8>,
    // max_stack: u16,
    // max_locals: u16,
    // code: Vec<u8>,
    // exception_table: Vec<u8>,
    // attributes: Vec<Attributes>,
}

#[derive(Default, Debug)]
pub struct Class {
    minor_version: u16,
    major_version: u16,
    constant_pool: Vec<Constant>,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    fields: Vec<Field>,
    methods: Vec<Method>,
    attributes: Vec<Attributes>,
}

#[derive(Debug)]
pub struct Frame<'a> {
    class: &'a mut Class,
    code: Vec<u8>,
    local_variables: Vec<i32>,
    operand_stack: Vec<i32>,
}
impl Class {
    pub fn new(classpath: String) -> Result<Class> {
        let mut buffer: Vec<u8> = Vec::new();
        BufReader::new(File::open(classpath)?).read_to_end(&mut buffer)?;

        let mut bytecode_reader = JavaBytecodeReader::new(buffer);
        assert_eq!(bytecode_reader.read_as_u64(4), 0xCAFEBABE);
        let minor_version: u16 = bytecode_reader.b2();
        let major_version: u16 = bytecode_reader.b2();
        let constant_pool_count: u16 = bytecode_reader.b2();
        let mut constant_pool: Vec<Constant> = Vec::new();
        for _ in 0..constant_pool_count - 1 {
            let tag = bytecode_reader.b1();
            match tag {
                0x1_u8 => {
                    let length = bytecode_reader.b2();
                    let data = bytecode_reader.read_as_string(length as usize);
                    constant_pool.push(Constant::Utf8(ConstantUtf8 { tag, data }));
                }
                0x7_u8 => {
                    let name_index = bytecode_reader.b2();
                    constant_pool.push(Constant::Class(ConstantClass { tag, name_index }));
                }
                0x8_u8 => {
                    let string_index = bytecode_reader.b2();
                    constant_pool.push(Constant::String(ConstantString { tag, string_index }));
                }
                0x9_u8 => {
                    let class_index = bytecode_reader.b2();
                    let name_and_type_index = bytecode_reader.b2();
                    constant_pool.push(Constant::Fieldref(ConstantFieldref {
                        tag,
                        class_index,
                        name_and_type_index,
                    }));
                }
                0xa_u8 => {
                    let class_index = bytecode_reader.b2();
                    let name_and_type_index = bytecode_reader.b2();
                    constant_pool.push(Constant::Methodref(ConstantMethodref {
                        tag,
                        class_index,
                        name_and_type_index,
                    }));
                }
                0xc_u8 => {
                    let name_index = bytecode_reader.b2();
                    let descriptor_index = bytecode_reader.b2();
                    constant_pool.push(Constant::NameAndType(ConstantNameAndType {
                        tag,
                        name_index,
                        descriptor_index,
                    }));
                }
                _ => panic!("Unsupported tag: {}.", tag),
            }
        }

        let access_flags = bytecode_reader.b2();
        let this_class = bytecode_reader.b2();
        let super_class = bytecode_reader.b2();

        let interfaces_count = bytecode_reader.b2();
        let mut interfaces: Vec<u16> = Vec::new();
        for _ in 0..interfaces_count {
            interfaces.push(bytecode_reader.b2());
        }

        let fields_count = bytecode_reader.b2();
        let mut fields: Vec<Field> = Vec::new();
        for _ in 0..fields_count {
            let access_flags = bytecode_reader.b2();

            let name_index = bytecode_reader.b2();
            let name = match constant_pool.index(name_index as usize - 1) {
                Constant::Utf8(c) => String::from(c.data.as_str()),
                _ => String::from(""),
            };

            let descriptor_index = bytecode_reader.b2();
            let descriptor = match constant_pool.index(descriptor_index as usize - 1) {
                Constant::Utf8(c) => String::from(c.data.as_str()),
                _ => String::from(""),
            };

            let attributes_count = bytecode_reader.b2();
            let mut attributes: Vec<Attributes> = Vec::new();
            for _ in 0..attributes_count {
                let attribute_name_index = bytecode_reader.b2();
                let attribute_name = match constant_pool.index(attribute_name_index as usize - 1) {
                    Constant::Utf8(c) => String::from(c.data.as_str()),
                    _ => String::from(""),
                };
                let attribute_length = bytecode_reader.b4();
                let info = bytecode_reader.read_as_vec(attribute_length as usize);

                attributes.push(Attributes {
                    attribute_name,
                    info,
                })
            }

            fields.push(Field {
                access_flags,
                name,
                descriptor,
                attributes,
            })
        }

        let methods_count = bytecode_reader.b2();
        let mut methods: Vec<Method> = Vec::new();
        for _ in 0..methods_count {
            let access_flags = bytecode_reader.b2();

            let name_index = bytecode_reader.b2();
            let name = match constant_pool.index(name_index as usize - 1) {
                Constant::Utf8(c) => String::from(c.data.as_str()),
                _ => String::from(""),
            };

            let descriptor_index = bytecode_reader.b2();
            let descriptor = match constant_pool.index(descriptor_index as usize - 1) {
                Constant::Utf8(c) => String::from(c.data.as_str()),
                _ => String::from(""),
            };

            let attributes_count = bytecode_reader.b2();
            let mut attributes: Vec<Attributes> = Vec::new();
            for _ in 0..attributes_count {
                let attribute_name_index = bytecode_reader.b2();
                let attribute_name = match constant_pool.index(attribute_name_index as usize - 1) {
                    Constant::Utf8(c) => String::from(c.data.as_str()),
                    _ => String::from(""),
                };
                let attribute_length = bytecode_reader.b4();
                let info = bytecode_reader.read_as_vec(attribute_length as usize);
                attributes.push(Attributes {
                    attribute_name,
                    info,
                })
            }

            methods.push(Method {
                access_flags,
                name,
                descriptor,
                attributes,
            })
        }

        let attributes_count = bytecode_reader.b2();
        let mut attributes: Vec<Attributes> = Vec::new();
        for _ in 0..attributes_count {
            let attribute_name_index = bytecode_reader.b2();
            let attribute_name = match constant_pool.index(attribute_name_index as usize - 1) {
                Constant::Utf8(c) => String::from(c.data.as_str()),
                _ => String::from(""),
            };
            let attribute_length = bytecode_reader.b4();
            let info = bytecode_reader.read_as_vec(attribute_length as usize);
            attributes.push(Attributes {
                attribute_name,
                info,
            })
        }

        Ok(Class {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    pub fn create_frame(&mut self, method_name: String, args: Vec<&str>) -> Frame {
        let mut method: Option<&Method> = Option::None;
        for m in &self.methods {
            if m.name == method_name {
                method = Some(m);
            }
        }
        let mut attribute: Option<&Attributes> = Option::None;
        for a in &method.unwrap().attributes {
            if a.attribute_name == "Code" {
                attribute = Some(a)
            }
        }
        let mut info = &attribute.unwrap().info;
        let mut info_reader = JavaBytecodeReader::new(info.to_vec());

        let max_stack = info_reader.b2();
        let max_locals = info_reader.b2();
        let code_lenght = info_reader.b4();
        let code = info_reader.read_as_vec(code_lenght as usize);
        let local_variables: Vec<i32> = args
            .into_iter()
            .map(|s| -> i32 { s.parse::<i32>().unwrap() })
            .collect::<Vec<i32>>();
        let operand_stack: Vec<i32> = Vec::new();

        Frame {
            class: self,
            code,
            local_variables,
            operand_stack,
        }
    }

    pub fn exec_main(&mut self, method_name: String, args: Vec<&str>) -> Option<i32> {
        let mut frame = self.create_frame(method_name, args);

        for code in frame.code {
            match code {
                26_u8 => {
                    frame.operand_stack.insert(0, frame.local_variables[0]);
                }
                27_u8 => {
                    frame.operand_stack.insert(0, frame.local_variables[1]);
                }
                96_u8 => {
                    let x = frame.operand_stack.pop()?;
                    let y = frame.operand_stack.pop()?;
                    frame.operand_stack.insert(0, x + y);
                }
                172_u8 => return frame.operand_stack.pop(),
                _ => panic!("Unsupported code: {}.", code),
            }
        }
        return Option::None;
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about = "A Very Simple Java Virtual Machine", long_about = None)]
struct Args {
    #[clap(long)]
    classpath: String,
    // #[clap(long)]
    // jar: String,
    method_name: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut class: Class = Class::new(args.classpath)?;
    let return_value = class.exec_main(args.method_name, vec!["1", "2"]);
    println!("{:#?}", return_value);

    Ok(())
}
