use crate::engines::rbatis_engine::node::Node;
use crate::engines::rbatis_engine::node::NodeType::{NString, NArg};
use serde_json::Value;
use serde_json::json;
use chrono::Local;
use crate::utils::time_util;
use serde_json::de::ParserNumber;
//use crate::engines::RustExpressionEngine::parser::{parser,  ParserTokens};
use crate::engines::rbatis_engine::runtime::{is_number, OptMap};
use std::collections::HashMap;
use std::collections::linked_list::LinkedList;
use crate::engines::rbatis_engine::{runtime, parser};
use std::rc::Rc;
//use test::Bencher;

#[derive(Clone, PartialEq)]
struct Eq<'a> {
    pub express: &'a str,
    pub eq: Value,
}

#[test]
fn TestNodeRun() {
    let john = json!({
        "a":1,
        "b":2,
        "c":"c",
        "d":null,
    });
    let expressions: Vec<Eq> = vec![
        Eq { express: "d.a == null", eq: json!(true) },
        Eq { express: "'2019-02-26' == '2019-02-26'", eq: json!(true) },
        Eq { express: "`f`+`s`", eq: json!("fs") },
        Eq { express: "a +1 > b * 8", eq: json!(false) },
        Eq { express: "a >= 0", eq: json!(true) },
        Eq { express: "'a'+c", eq: json!("ac") },
        Eq { express: "b", eq: json!(2) },
        Eq { express: "a < 1", eq: json!(false) },
        Eq { express: "a +1 > b*8", eq: json!(false) },
        Eq { express: "a * b == 2", eq: json!(true) },
        Eq { express: "a - b == 0", eq: json!(false) },
        Eq { express: "a >= 0 && a != 0", eq: json!(true) },
        Eq { express: "a == 1 && a != 0", eq: json!(true) },
        Eq { express: "1 > 3 ", eq: json!(false) },
        Eq { express: "1 + 2 != nil", eq: json!(true) },
        Eq { express: "1 != null", eq: json!(true) },
        Eq { express: "1 + 2 != nil && 1 > 0 ", eq: json!(true) },
        Eq { express: "1 + 2 != nil && 2 < b*8 ", eq: json!(true) },
    ];


    let mut index = 0;
    for item in expressions {
        println!("{}", item.express.clone());
        //TODO let parserArray = parser(item.to_string(), &OptMap::new());

        let mut boxNode = parser::parser(item.express.to_string(), &OptMap::new()).unwrap();
        let result = boxNode.eval(&john).unwrap();
        println!("express: {} >>>>> {}", item.express,&result);
        let resultValue = &item.eq.clone();
        if !result.eq(resultValue) {
            // println!("exe express fail:".to_owned()+item);
            panic!("[rbatis] >>>>>>>>>>>>>>>>>>>>>exe fail express:'".to_owned() + item.clone().express + "'");
        }
        index += 1;
    }
}


#[test]
fn TestStringNode() {
    let mut strNode = Node::new_string("sadf");
    strNode.eval(&Value::Null {});
    //println!("value:{}", result);
}

#[test]
fn TestArgNode() {
    let john = json!({
        "name": "John Doe",
        "age": Value::Null,
         "sex":{
            "a":"i'm a",
            "b":"i'm b",
         },
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });

    let mut argNode = Node::new_arg("sex.a");
    argNode.eval(&john);
    //println!("value:{},error:{}", result, Error);
}

#[test]
fn benchmark_arg_node() {
    let john = json!({
        "name": "John Doe",
        "age": Value::Null,
         "sex":{
            "a":"i'm a",
            "b":"i'm b",
         },
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });

    let mut argNode = Node::new_arg("sex.a");

    let total = 100000;
    let now = Local::now();
    for i in 0..total {
        argNode.eval(&john);
    }
    time_util::count_time(total, now);
    time_util::count_tps(total, now);
}

#[test]
fn TestNumberNode() {
    let john = json!({
        "name": "John Doe",
        "age": 1,
         "sex":{
            "a":"i'm a",
            "b":"i'm b",
         },
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });
    let mut numb = Node::new_number_f64(1.02 as f64);
    numb.eval(&john);
    // println!("{}", value);
}

#[test]
fn benchmark_parser_token() {
    let s = "'2019-02-26' == '2019-02-26'".to_string();
    let optMap = OptMap::new();

    let total = 100000;
    let now = Local::now();
    for i in 0..total {
        runtime::parser_tokens(&s, &optMap);
    }
    time_util::count_time(total, now);
    time_util::count_tps(total, now);
}


//#[bench]
//fn bench_node_eval(b: &mut Bencher) {
//    let rc=Rc::new("asdf".to_string());
//    b.iter(|| {
//        rc.clone();
//    });
//}