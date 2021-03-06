#[macro_use]
extern crate mlua_derive;

use mlua::prelude::*;
use cidr::AnyIpCidr;
use std::str::FromStr;
use std::cmp::Ordering;
use md5;

pub mod avltree;
use avltree::AVLTreeMap;


#[derive(Debug)]
struct IPSet (AVLTreeMap<AnyIpCidr,String>);


fn cmp(net:&AnyIpCidr, ip:&AnyIpCidr) -> Ordering {
    let eq = match (ip.first_address(), ip.last_address(), net.first_address(), net.last_address()) {
        (Some(kf), Some(kl), Some(vf), Some(vl)) => ((kf <= vl) && (vf <= kl)),
        (_, _, None, None) => true,
        (None, None, Some(_), Some(_)) => true,
        (_,_,_,_) => panic!("internal error")
    };
    if eq {
        return Ordering::Equal
    }
    else {
        net.cmp(ip)
    }
}


impl IPSet {
    pub fn new() -> IPSet {
        IPSet(AVLTreeMap::new())
    }

    pub fn contains(&self, key: &AnyIpCidr) -> bool {
        self.0.contains_custom(key, cmp)
   }

    pub fn get(&self, key: &AnyIpCidr) -> Option<&String> {
        self.0.get_custom(key, cmp)
   }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, key: AnyIpCidr, value: String) -> bool {
        self.0.insert(key, value)
    }
}


impl mlua::UserData for IPSet {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("len",
                           |_, this:&IPSet, _:()| {
                               Ok(this.len()) 
                           }
        );
        methods.add_method("contains",
                           |_, this:&IPSet, value:String| {
                               let a:AnyIpCidr = AnyIpCidr::from_str(&value).unwrap();
                               Ok(this.contains(&a))
                           }
        );
        methods.add_method("get",
                           |_, this:&IPSet, value:String| {
                               let a:AnyIpCidr = AnyIpCidr::from_str(&value).unwrap();
                               match this.get(&a) {
                                   Some(v) => Ok(Some(v.clone())),
                                   None => Ok(None),
                               }
                           }
        );
        methods.add_method_mut("add",
                               |_, this:&mut IPSet, (key,value): (String,String)| {
                                   let a:AnyIpCidr = AnyIpCidr::from_str(&key).unwrap();
                                   this.insert(a, value);
                                   Ok(())
                               })
            ;
    }
}



fn new_ip_set(_: &Lua, _:()) -> LuaResult<IPSet> {
    let ipset = IPSet::new();
    Ok(ipset)
}


fn modhash(_: &Lua, (val,m):(String,u32)) -> LuaResult<Option<u32>> {
    let digest = md5::compute(val);

    let res = match m {
        0 => None,
        m => {
            let h:u128 = u128::from_be_bytes(*digest);
            Some((h % (m as u128)) as u32)
        },
    };
    Ok(res)
}




#[lua_module]
fn iptools(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("new_ip_set", lua.create_function(new_ip_set)?)?;
    exports.set("modhash", lua.create_function(modhash)?)?;
    Ok(exports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipset() {
        let mut ipset= IPSet::new();

        let ip1 = ["1.2.3.0/24", "1.1.1.1/32", "192.168.8.0/24","10.0.0.0/8"];
        let ip2 = ["192.168.8.24", "192.168.0.0/16", "192.168.8.64/29", "1.2.2.0/23", "10.1.2.3", "10.0.0.0/7","0.0.0.0/0"];
        let ip3 = ["1.2.4.0/24", "1.1.1.2/32", "192.168.7.0/24", "2.0.0.0/8"];

        for n in ip1.iter() {
            let a = AnyIpCidr::from_str(&n).unwrap();
            println!("Inserting {:?}",a);
            ipset.0.insert(a,"hello".to_string());
        }
        assert!(ipset.len() == ip1.len());


        for n in ip1.iter() {
            let a = AnyIpCidr::from_str(&n).unwrap();
            println!("Testing {:?}",a);
            assert!(ipset.contains(&a));
        }

        for n in ip2.iter() {
            let a = AnyIpCidr::from_str(&n).unwrap();
            println!("Testing {:?}",a);
            assert!(ipset.contains(&a));
        }

        for n in ip3.iter() {
            let a = AnyIpCidr::from_str(&n).unwrap();
            println!("Testing {:?}",a);
            assert!(!ipset.contains(&a));
        }
    }
}
