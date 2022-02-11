use std::sync::{Arc, RwLock, Mutex};
use crate::internal_coms::{BusKey};
use crate::config_loader::{CfgKeybind};
use crate::{internal_coms, subkeybind};
use std::thread;

extern crate timer;
extern crate chrono;

/*
this struct will hold communications channel between the manager and the subkeybind workers and the matching timer
 */

struct ManagerWorkSpace{
    //need the coms channel
    coms: Arc<Mutex<bool>>,
    //put a timer
    timer: timer::Timer,
    //put the timer guard, to cancel at any time
    guard: Option<timer::Guard>,
    //and now the signal, on or off
    signal: Arc<Mutex<bool>>,
}

impl ManagerWorkSpace{
    pub fn new(coms: Arc<Mutex<bool>> ) ->ManagerWorkSpace{
        ManagerWorkSpace{
            coms,
            timer: timer::Timer::new(),
            guard: None,
            signal: Arc::new(Mutex::new(false)),
        }
    }

}




pub fn new(config: CfgKeybind, master_bus: Arc<RwLock<Vec<BusKey>>>){
    let mut sub_keybind_management = vec![];
    config.sub_keybinds.into_iter().for_each(|config|{
        let arc_link_master = master_bus.clone();
        let arc_link_subkeybind = internal_coms::ManagerKeybindsComs::new().generate_arc_link();
        let arc_link_manager = arc_link_subkeybind.clone();
        let name = format!("sub_keybinds{}_{}", config.key_code, config.key_state);

        thread::Builder::new().name(name).spawn(move || {
            subkeybind::start(config, arc_link_subkeybind.clone(), arc_link_master)
            }).unwrap();
        sub_keybind_management.push(ManagerWorkSpace::new(arc_link_manager))
    });
    let mut are_all_keylistener_matched: u8;
    let threshold = config.timer_threshold.clone();
    loop {
        //check all coms
        sub_keybind_management.iter_mut().for_each(|mut workspace|{
            if *workspace.coms.lock().unwrap() == true{
                //if one sub_keybind call a matched set the whole to true
                *workspace.signal.lock().unwrap()=true;
                *workspace.coms.lock().unwrap()=false;//the signal has been acknowledged set
                match &workspace.guard{
                    Some(p) => drop(p),
                    _ => {}
                }
                let lock_for_timer= workspace.signal.clone();
                //we drop the previous timer because in case of the person trigger the button before the timing end;
                workspace.guard = Some(workspace.timer.schedule_with_delay(chrono::Duration::milliseconds(threshold as i64), move ||{
                    *lock_for_timer.lock().unwrap()=false;
                }))
            }
        });
        //check all matched
        let counts_of_sub_keybinds = sub_keybind_management.len(); // get the number of elements inside the vector to compare if all said it matched
        are_all_keylistener_matched=0;
        sub_keybind_management.iter().for_each(|is_matched|{
            if *is_matched.signal.lock().unwrap() == true{
                are_all_keylistener_matched+=1;
            }
        });
        if are_all_keylistener_matched == counts_of_sub_keybinds as u8 {
            println!("{}", config.adr_name);
            turn_all_signal_to_false(&mut sub_keybind_management);
        }
    }
}
fn turn_all_signal_to_false(vec: &mut Vec<ManagerWorkSpace>){
    vec.iter_mut().for_each(|lock|{
        *lock.signal.lock().unwrap() = false;
    })
}
