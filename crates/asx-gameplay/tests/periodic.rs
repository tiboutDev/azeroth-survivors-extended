use asx_gameplay::{periodic::Batch, spells::{Events, Healing}, support::Spell};
#[test]
fn personal_foreign_and_mixed_totals_preserve_native_order() {
    let mut b = Batch::default(); b.observe(4., false).unwrap(); b.observe(8., true).unwrap();
    let p = b.plan(5., 7.).unwrap();
    assert_eq!((p.raw_total,p.foreign_raw,p.personal_raw,p.personal_rows,p.foreign_rows),(12.,8.,4.,1,1));
    let mut foreign = Batch::default(); foreign.observe(12., true).unwrap(); assert_eq!(foreign.plan(5.,7.).unwrap().personal_raw,0.);
    let mut own = Batch::default(); own.observe(12.,false).unwrap(); assert_eq!(own.plan(5.,7.).unwrap().personal_raw,12.);
    assert_eq!(b.plan(5.,7.).unwrap(),p,"planning is pure until adapter consumes its scope");
}
#[test]
fn rounding_overshoot_and_tiny_negative_overheal_follow_existing_f32_math() {
    for (effective,overheal,raw) in [(64.,-31.,33.),(0.1,0.2,0.3),(1e8,1.,1e8)] {
        let mut b = Batch::default(); b.observe(raw,true).unwrap();
        assert_eq!(b.plan(effective,overheal).unwrap().personal_raw.to_bits(),((effective+overheal-raw).max(0.)as f32).to_bits());
    }
}
#[test]
fn invalid_values_and_overfull_batches_fail_before_mutation() {
    let mut b = Batch::default();b.observe(2.,true).unwrap();let before=b.plan(1.,1.).unwrap();
    for raw in [f32::NAN,f32::INFINITY,-1.,1e13] {assert!(b.observe(raw,true).is_err());assert_eq!(b.plan(1.,1.).unwrap(),before);}
    for (effective,overheal) in [(f32::NAN,1.),(1.,f32::INFINITY),(-1.,1.)] {assert!(b.plan(effective,overheal).is_err());}
    for _ in 1..512 {b.observe(1.,false).unwrap();} assert!(b.observe(1.,false).is_err());
}
#[test]
fn own_periodic_heals_are_not_foreign_and_budget_events_do_not_complete_casts() {
    let mut e=Events::default();let id=e.begin(1,2,Spell::Renew).unwrap();
    let c=e.healing(Healing{caster:1,target:1,spell:Spell::Renew,raw:5.,effective:2.,periodic:true}).unwrap();assert_eq!(c.foreign_periodic,0.);
    let mut batch=Batch::default();batch.observe(5.,false).unwrap();e.periodic_budget(&batch,2.,3.).unwrap();
    assert_eq!((e.periodic_batches,e.completed,e.active()),(1,0,1));assert!(e.get(id).is_some());
    assert!(e.periodic_budget(&batch,f32::NAN,0.).is_err());assert_eq!(e.periodic_batches,1);
}
