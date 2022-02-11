
//this trait will in function of the object make the correct matching it's a bit hard to understand even to myself tbh
pub trait KeyMatching{
    fn key_matching(&mut self, key_code: u16, key_value: u32) -> bool;
}


struct Simple{
    key_code: u16,
    key_value: u32,
}

//simply compare the config with what kernel say, really simple
impl KeyMatching for Simple{
    fn key_matching(&mut self, key_code: u16, key_value: u32) -> bool {
        if self.key_code == key_code && self.key_value == key_value{
            return true
        }
        return false
    }
}
/*
This Struct will be be for press that have to be X time long before matching
The idea is when a 1 is matched it init the
 */
use std::time::{Duration,Instant};

struct LongPress{
    key_code: u16,
    timer_threshold: Duration,
    start_timer: Instant,
}

impl LongPress {
    fn new(key_code: u16, threshold: u64) ->LongPress{
        LongPress {
            key_code,
            timer_threshold: Duration::from_millis(threshold),
            start_timer: Instant::now()
        }
    }
}

impl KeyMatching for LongPress{
    fn key_matching(&mut self, key_code: u16, key_value: u32) -> bool {
        if self.key_code == key_code{
            //we press down the keyboard key
            if key_value == 1{
                self.start_timer=Instant::now();
            }
            /* it's else 2 a kernel hold down or a release
               Here is a tricky part, this system cannot hold very precise time length
               because at each "kernel tic" this function will be invoked and kernel tics aren't that frequent.
               But to fix this i would need a total rewrite
             */
            else{
                let now = Instant::now();
                // just check if the time elapsed between the first press and the second press is more than the time we wanted
                if now.duration_since(self.start_timer) > self.timer_threshold {
                    return true;
                }
                //the press wasn't long enough
                return false;
            }
        }
        return false
    }
}
/*
to explain in short, this function
in our config file we have humans understandable key_states such as release, press, hold and tons of other options
so what i do is to transform those Strings into key value, it's more of a key_code_function to be franc
i think that for the sake of the developer it's better to make matching with 1 referring to a certain function/type than words

note i rather u32 than i32 because i don't like to deal with negative numbers
However in the kernel the key_value parameter if i can call that that way is a i32
So i might be wrong but i'll stick to positive integer only

This is where you can link a key_state to a key value
*/

pub(crate) fn trans_key_state_to_key_value(cfg_key_state: String)->u32{//u32 because since it's related to the 4bytes key_value of the kernel input event system
    /*

    for the simple struct example, the function will simply match the key code sent by the kernel and the one converted
    but for other keymatching more complex it may not matter
    */

    return match cfg_key_state.as_str() {
        "press" => 1,
        ">" => 1,
        "release" => 0,
        "<" => 0,
        "hold" => 2,
        "_" => 2,
        "longpress" => 2,
        _ => 1,
    }
}
/*
This function is were you link your in-config key state names into what kind of implementation of the keymatching trait

 */
pub(crate) fn trans_key_state_to_key_function(cfg_key_state: String)->u8{
    return match cfg_key_state.as_str() {
        "press" => 1,
        ">" => 1,
        "release" => 1,
        "<" => 1,
        "hold" => 1,
        "_" => 1,
        _ => 1,
    }
}



/*
This is were in function of what you linked previously you will generate the right implementation
 */
pub fn new(cfg_key_code: u16, cfg_key_state: String, cfg_longpress_threshold: Option<u64>) -> Box<dyn KeyMatching>{
    let cfg_key_value = trans_key_state_to_key_value(cfg_key_state.clone());
    let code= trans_key_state_to_key_function(cfg_key_state);
    return match code{
        1 => Box::new(Simple{key_code: cfg_key_code, key_value: cfg_key_value}),
        2 => Box::new(LongPress::new(cfg_key_code, cfg_longpress_threshold.unwrap())),
        _ => Box::new(Simple{key_code: cfg_key_code, key_value: cfg_key_value}),//better make a panic tbh
    }
}