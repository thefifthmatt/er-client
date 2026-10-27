use eldenring::cs::{
    CSFeManImp, PlayerIns,
};
use fromsoftware_shared::{FromStatic, Program};
use ilhook::x64::*;
use pelite::pe::Pe;

use crate::rva;

/// Allow opening map in combat only if not getting damaged and no boss fight.
fn should_allow_combat_map() -> bool {
    // This doesn't capture all damage, just staggers, but good enough for this.
    if let Ok(player) = unsafe { PlayerIns::local_player() }
        && player.chr_ins.modules.action_flag.damage_level > 0 {
        return false;
    }
    if let Ok(fe_man) = unsafe { CSFeManImp::instance() }
        && fe_man.boss_health_displays.iter().any(|health| health.fmg_id > 0) {
        return false;
    }
    true
}

/// Hooks misc interactions not related to item or location state.
pub unsafe fn hook_misc() {
    let program = Program::current();
    let rvas = rva::get();
    let check_combat_open_map_addr = program.rva_to_va(rvas.check_combat_open_map).unwrap();
    let check_combat_close_map_addr = program.rva_to_va(rvas.check_combat_close_map).unwrap();

    let callback = |reg: *mut Registers| {
        let in_combat = unsafe { (*reg).rax as u8 } != 0;
        if in_combat && should_allow_combat_map() {
            unsafe { (*reg).rax = 0; }
        }
    };
    std::mem::forget(
        unsafe {
            hook_closure_jmp_back(
                check_combat_open_map_addr as usize,
                callback,
                CallbackOption::None,
                HookFlags::empty(),
            )
        }
        .expect("Hooking check_combat_open_map_addr failed"),
    );
    std::mem::forget(
        unsafe {
            hook_closure_jmp_back(
                check_combat_close_map_addr as usize,
                callback,
                CallbackOption::None,
                HookFlags::empty(),
            )
        }
        .expect("Hooking check_combat_close_map_addr failed"),
    );
}
