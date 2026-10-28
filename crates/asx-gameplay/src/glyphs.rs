//! Run-local spell specializations. No native layouts or random global state.
use std::collections::BTreeMap;
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {SweepingCleave,HeavyCleave,WideVortex,DenseVortex,LingeringWounds,RapidBleeding,RollingThunder,LastingThunder,BroadStrike,HeavyMortal,SweepingExecution,FinalVerdict,LingeringDread,PiercingDread,LastingRally,FierceRally,HungeringStrike,FrenziedThirst,BatteringShield,CrushingShield,SpreadingRage,FocusedRage,HallowedGround,LingeringConsecration,ForkedExorcism,SolemnExorcism,SwiftWrath,WeightOfWrath,RadiantShock,SearingShock,EncirclingLight,ZealousWrath,SharedJudgement,Condemnation,LastingJustice,SwiftJustice,DeepPrayer,QuickPrayer,EnduringSanctuary,FleetingSanctuary,BarbedAegis,LastingAegis,SpreadingRedemption,BrilliantRedemption,ScatteredArrows,HeavyArrows,ArcaneSplinters,ArcaneOverload,Broadside,RapidSalvo,LingeringVenom,VirulentSpread,BlanketingVolley,SustainedVolley,WideDetonation,VolatileCharge,LingeringConcussion,ScatteredConcussion,SplitAim,PatientAim,SpreadingFlames,EnduringEmbers,WideFrost,LingeringFrost,FarReachingMark,PotentMark,SweepingBlades,HeavyCuts,SpreadingExecution,DeepExecution,WhirlingBlades,ConcentratedFlurry,LingeringRupture,SpreadingWounds,LastingSuppression,SwiftSuppression,SweepingGouge,LingeringDaze,SpectralGuard,RecklessStrike,EnduringRush,IntenseRush,ForkedBlades,PatientStrike,LastingEdge,KeenEdge,SharedFate,CruelFate,ScatteredFaith,SwiftPrayer,LingeringPain,SpreadingAgony,SplinteredMind,CrushingThought,RadiantNova,MendingNova,DeepShelter,FleetingShelter,DeepRenewal,SwiftRenewal,SearingFaith,RestlessFlame,CrushingTorment,RelentlessTorment,EchoingTerror,SpreadingTerror,SpreadingPlague,RavenousPlague,RadiantWell,EnduringLight,ForkedLightning,RestlessStorm,FarReachingStorm,OverchargedChain,ShatteringEarth,CrushingEarth,SpreadingEmbers,SearingBrand,LingeringShock,ShatteringFrost,OverchargedShield,VolatileShield,SpreadingMagma,RoaringMagma,ForkedEmbers,BlazingEmbers,ExpandingNova,IntenseNova,DeepWaters,SwiftWaters,SpreadingPyre,DevouringPyre,ForkedFlame,RestlessFire,LingeringChill,ShatteringIce,ArcaneBarrage,RapidChanneling,DeepFreeze,SpreadingFrost,ExpandingArcana,DenseArcana,WideBlizzard,BitterBlizzard,SpreadingInferno,RoaringInferno,ScatteredSparks,OverchargedSpark,LingeringWinter,ShatteringWinter,ExpandingWave,LingeringShockwave,TwinPyres,HeavyPyre,DeepBarrier,FleetingBarrier,FlockMaster,RestlessFlock,ForkedShadows,RestlessShadows,SpreadingDecay,RestlessDecay,SpreadingImmolation,SearingImmolation,SpreadingCurse,CrushingAgony,WideInferno,DenseInferno,ForkedCoil,RavenousCoil,ScatteredTorment,IntenseTorment,SharedHunger,VoraciousDrain,SpreadingDread,LingeringTerror,TwinSouls,HungrySouls,UnboundPact,EnduringPact,ForkedWrath,HeavyWrath,SpreadingMoonlight,SearingMoonlight,TwinStars,FallingStar,WideTempest,ViolentTempest,SpreadingSwarm,RavenousSwarm,Overgrowth,DeepRoots,AbundantGrowth,PersistentGrowth,FlourishingRegrowth,SwiftRegrowth,BarbedThorns,LastingThorns,SweepingClaws,SavageClaws,RapidShredding,RendingClaws,FarReachingSparks,PotentSparks }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Modifiers {pub damage:f32,pub cooldown:f32,pub targets:u32,pub radius:f32,pub duration:f32,pub health_damage:f32,pub effect:f32,pub healing:f32,pub proc_rate:f32}
const NEUTRAL:Modifiers=Modifiers{damage:1.,cooldown:1.,targets:0,radius:1.,duration:1.,health_damage:0.,effect:1.,healing:1.,proc_rate:1.};
impl Default for Modifiers{fn default()->Self{NEUTRAL}}
struct Profile{spell:u32,name:&'static str,lines:[&'static str;2],factors:Modifiers}
const PROFILES:[Profile;204]=[
 Profile{spell:845,name:"Sweeping Cleave",lines:["+2 targets","-25% damage"],factors:Modifiers{damage:0.75,targets:2,..NEUTRAL}},
 Profile{spell:845,name:"Heavy Cleave",lines:["+40% damage","50% longer cooldown"],factors:Modifiers{damage:1.4,cooldown:1.5,..NEUTRAL}},
 Profile{spell:1680,name:"Wide Vortex",lines:["+40% radius","-25% damage"],factors:Modifiers{damage:0.75,radius:1.4,..NEUTRAL}},
 Profile{spell:1680,name:"Dense Vortex",lines:["+40% damage","-30% radius"],factors:Modifiers{damage:1.4,radius:0.7,..NEUTRAL}},
 Profile{spell:772,name:"Lingering Wounds",lines:["+50% bleed duration","-25% bleed DPS"],factors:Modifiers{damage:0.75,duration:1.5,..NEUTRAL}},
 Profile{spell:772,name:"Rapid Bleeding",lines:["+50% bleed DPS","-50% bleed duration"],factors:Modifiers{damage:1.5,duration:0.5,..NEUTRAL}},
 Profile{spell:6343,name:"Rolling Thunder",lines:["+40% radius","-25% damage"],factors:Modifiers{damage:0.75,radius:1.4,..NEUTRAL}},
 Profile{spell:6343,name:"Lasting Thunder",lines:["+50% slow duration","35% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.35,..NEUTRAL}},
 Profile{spell:12294,name:"Broad Strike",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:12294,name:"Heavy Mortal Strike",lines:["+45% damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:5308,name:"Sweeping Execution",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:5308,name:"Final Verdict",lines:["+45% damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:1160,name:"Lingering Dread",lines:["+50% debuff duration","40% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.4,..NEUTRAL}},
 Profile{spell:1160,name:"Piercing Dread",lines:["+40% weakening strength","-30% radius"],factors:Modifiers{effect:1.4,radius:0.7,..NEUTRAL}},
 Profile{spell:6673,name:"Lasting Rally",lines:["+50% buff duration","40% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.4,..NEUTRAL}},
 Profile{spell:6673,name:"Fierce Rally",lines:["+40% damage buff strength","-35% buff duration"],factors:Modifiers{effect:1.4,duration:0.65,..NEUTRAL}},
 Profile{spell:23881,name:"Hungering Strike",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:23881,name:"Frenzied Thirst",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:23922,name:"Battering Shield",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:23922,name:"Crushing Shield",lines:["+40% damage","-40% stun duration"],factors:Modifiers{damage:1.4,effect:0.6,..NEUTRAL}},
 Profile{spell:2687,name:"Spreading Rage",lines:["+40% eruption radius","-25% eruption damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:2687,name:"Focused Rage",lines:["+40% eruption damage","-30% eruption radius"],factors:Modifiers{damage:1.4,radius:0.7,..NEUTRAL}},
 Profile{spell:26573,name:"Hallowed Ground",lines:["+40% radius","-25% periodic damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:26573,name:"Lingering Consecration",lines:["+50% duration","-25% periodic damage"],factors:Modifiers{duration:1.5,damage:0.75,..NEUTRAL}},
 Profile{spell:879,name:"Forked Exorcism",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:879,name:"Solemn Exorcism",lines:["+45% damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:24275,name:"Swift Wrath",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:24275,name:"Weight of Wrath",lines:["+50% damage","60% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.6,..NEUTRAL}},
 Profile{spell:25912,name:"Radiant Shock",lines:["+50% healing","-30% damage"],factors:Modifiers{healing:1.5,damage:0.7,..NEUTRAL}},
 Profile{spell:25912,name:"Searing Shock",lines:["+40% damage","-40% healing"],factors:Modifiers{damage:1.4,healing:0.6,..NEUTRAL}},
 Profile{spell:2812,name:"Encircling Light",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:2812,name:"Zealous Wrath",lines:["30% shorter cooldown","-30% radius"],factors:Modifiers{cooldown:0.7,radius:0.7,..NEUTRAL}},
 Profile{spell:20271,name:"Shared Judgement",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:20271,name:"Condemnation",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:853,name:"Lasting Justice",lines:["+50% stun duration","-35% damage"],factors:Modifiers{effect:1.5,damage:0.65,..NEUTRAL}},
 Profile{spell:853,name:"Swift Justice",lines:["30% shorter cooldown","-40% stun duration"],factors:Modifiers{cooldown:0.7,effect:0.6,..NEUTRAL}},
 Profile{spell:635,name:"Deep Prayer",lines:["+40% healing","50% longer cooldown"],factors:Modifiers{healing:1.4,cooldown:1.5,..NEUTRAL}},
 Profile{spell:635,name:"Quick Prayer",lines:["30% shorter cooldown","-40% healing"],factors:Modifiers{cooldown:0.7,healing:0.6,..NEUTRAL}},
 Profile{spell:642,name:"Enduring Sanctuary",lines:["+50% duration","60% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.6,..NEUTRAL}},
 Profile{spell:642,name:"Fleeting Sanctuary",lines:["30% shorter cooldown","-40% duration"],factors:Modifiers{cooldown:0.7,duration:0.6,..NEUTRAL}},
 Profile{spell:20925,name:"Barbed Aegis",lines:["+40% retaliation damage","-35% duration"],factors:Modifiers{damage:1.4,duration:0.65,..NEUTRAL}},
 Profile{spell:20925,name:"Lasting Aegis",lines:["+50% duration","-25% retaliation damage"],factors:Modifiers{duration:1.5,damage:0.75,..NEUTRAL}},
 Profile{spell:7328,name:"Spreading Redemption",lines:["+40% burst radius","-25% burst damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:7328,name:"Brilliant Redemption",lines:["+50% burst damage","-35% burst radius"],factors:Modifiers{damage:1.5,radius:0.65,..NEUTRAL}},
 Profile{spell:75,name:"Scattered Arrows",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:75,name:"Heavy Arrows",lines:["+40% damage","50% longer cooldown"],factors:Modifiers{damage:1.4,cooldown:1.5,..NEUTRAL}},
 Profile{spell:3044,name:"Arcane Splinters",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:3044,name:"Arcane Overload",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:2643,name:"Broadside",lines:["+3 targets","-30% damage"],factors:Modifiers{targets:3,damage:0.7,..NEUTRAL}},
 Profile{spell:2643,name:"Rapid Salvo",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:1978,name:"Lingering Venom",lines:["+50% duration","-25% periodic damage"],factors:Modifiers{duration:1.5,damage:0.75,..NEUTRAL}},
 Profile{spell:1978,name:"Virulent Spread",lines:["+2 targets","-30% periodic damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:1510,name:"Blanketing Volley",lines:["+40% radius","-25% periodic damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:1510,name:"Sustained Volley",lines:["+50% duration","60% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.6,..NEUTRAL}},
 Profile{spell:13812,name:"Wide Detonation",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:13812,name:"Volatile Charge",lines:["+50% damage","-30% radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:5116,name:"Lingering Concussion",lines:["+50% slow duration","40% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.4,..NEUTRAL}},
 Profile{spell:5116,name:"Scattered Concussion",lines:["+2 targets","-35% slow duration"],factors:Modifiers{targets:2,duration:0.65,..NEUTRAL}},
 Profile{spell:19434,name:"Split Aim",lines:["+1 target","-35% damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:19434,name:"Patient Aim",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:13797,name:"Spreading Flames",lines:["+40% radius","-25% periodic damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:13797,name:"Enduring Embers",lines:["+50% duration","60% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.6,..NEUTRAL}},
 Profile{spell:13810,name:"Wide Frost",lines:["+40% zone radius","-30% zone duration"],factors:Modifiers{radius:1.4,duration:0.7,..NEUTRAL}},
 Profile{spell:13810,name:"Lingering Frost",lines:["+50% zone duration","-30% zone radius"],factors:Modifiers{duration:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:1130,name:"Far-Reaching Mark",lines:["+40% explosion radius","-25% explosion damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:1130,name:"Potent Mark",lines:["+50% explosion damage","-30% explosion radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:1752,name:"Sweeping Blades",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:1752,name:"Heavy Cuts",lines:["+40% damage","50% longer cooldown"],factors:Modifiers{damage:1.4,cooldown:1.5,..NEUTRAL}},
 Profile{spell:2098,name:"Spreading Execution",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:2098,name:"Deep Execution",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:13877,name:"Whirling Blades",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:13877,name:"Concentrated Flurry",lines:["+45% damage","-30% radius"],factors:Modifiers{damage:1.45,radius:0.7,..NEUTRAL}},
 Profile{spell:1943,name:"Lingering Rupture",lines:["+50% duration","-25% periodic damage"],factors:Modifiers{duration:1.5,damage:0.75,..NEUTRAL}},
 Profile{spell:1943,name:"Spreading Wounds",lines:["+2 targets","-30% periodic damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:408,name:"Lasting Suppression",lines:["+50% stun duration","50% longer cooldown"],factors:Modifiers{effect:1.5,cooldown:1.5,..NEUTRAL}},
 Profile{spell:408,name:"Swift Suppression",lines:["30% shorter cooldown","-40% stun duration"],factors:Modifiers{cooldown:0.7,effect:0.6,..NEUTRAL}},
 Profile{spell:1776,name:"Sweeping Gouge",lines:["+2 targets","-35% stun duration"],factors:Modifiers{targets:2,effect:0.65,..NEUTRAL}},
 Profile{spell:1776,name:"Lingering Daze",lines:["+50% stun duration","-35% damage"],factors:Modifiers{effect:1.5,damage:0.65,..NEUTRAL}},
 Profile{spell:14278,name:"Spectral Guard",lines:["+40% shield per target","-30% damage"],factors:Modifiers{effect:1.4,damage:0.7,..NEUTRAL}},
 Profile{spell:14278,name:"Reckless Strike",lines:["+45% damage","-40% shield per target"],factors:Modifiers{damage:1.45,effect:0.6,..NEUTRAL}},
 Profile{spell:13750,name:"Enduring Rush",lines:["+50% duration","50% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.5,..NEUTRAL}},
 Profile{spell:13750,name:"Intense Rush",lines:["+40% cooldown recovery bonus","-35% duration"],factors:Modifiers{effect:1.4,duration:0.65,..NEUTRAL}},
 Profile{spell:53,name:"Forked Blades",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:53,name:"Patient Strike",lines:["+45% damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:5171,name:"Lasting Edge",lines:["+50% duration","50% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.5,..NEUTRAL}},
 Profile{spell:5171,name:"Keen Edge",lines:["+40% damage buff strength","-35% duration"],factors:Modifiers{effect:1.4,duration:0.65,..NEUTRAL}},
 Profile{spell:14186,name:"Shared Fate",lines:["+1 triggered target","-35% triggered damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:14186,name:"Cruel Fate",lines:["+50% triggered damage","-30% maximum combo point rate"],factors:Modifiers{damage:1.5,proc_rate:0.7,..NEUTRAL}},
 Profile{spell:585,name:"Scattered Faith",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:585,name:"Swift Prayer",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:589,name:"Lingering Pain",lines:["+50% duration","-25% periodic damage"],factors:Modifiers{duration:1.5,damage:0.75,..NEUTRAL}},
 Profile{spell:589,name:"Spreading Agony",lines:["+2 targets","-30% periodic damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:8092,name:"Splintered Mind",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:8092,name:"Crushing Thought",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:15237,name:"Radiant Nova",lines:["+40% damage radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:15237,name:"Mending Nova",lines:["+50% healing","-30% damage"],factors:Modifiers{healing:1.5,damage:0.7,..NEUTRAL}},
 Profile{spell:17,name:"Deep Shelter",lines:["+40% absorption","60% longer cooldown"],factors:Modifiers{effect:1.4,cooldown:1.6,..NEUTRAL}},
 Profile{spell:17,name:"Fleeting Shelter",lines:["30% shorter cooldown","-40% absorption"],factors:Modifiers{cooldown:0.7,effect:0.6,..NEUTRAL}},
 Profile{spell:139,name:"Deep Renewal",lines:["+40% total healing","50% longer cooldown"],factors:Modifiers{healing:1.4,cooldown:1.5,..NEUTRAL}},
 Profile{spell:139,name:"Swift Renewal",lines:["30% shorter cooldown","-40% total healing"],factors:Modifiers{cooldown:0.7,healing:0.6,..NEUTRAL}},
 Profile{spell:14914,name:"Searing Faith",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:14914,name:"Restless Flame",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:15407,name:"Crushing Torment",lines:["+40% damage","-40% slow duration"],factors:Modifiers{damage:1.4,duration:0.6,..NEUTRAL}},
 Profile{spell:15407,name:"Relentless Torment",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:8122,name:"Echoing Terror",lines:["+50% fear duration","50% longer cooldown"],factors:Modifiers{effect:1.5,cooldown:1.5,..NEUTRAL}},
 Profile{spell:8122,name:"Spreading Terror",lines:["+40% radius","-30% fear duration"],factors:Modifiers{radius:1.4,effect:0.7,..NEUTRAL}},
 Profile{spell:2944,name:"Spreading Plague",lines:["+2 targets","-30% periodic damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:2944,name:"Ravenous Plague",lines:["+45% periodic damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:724,name:"Radiant Well",lines:["+1 target per pulse","-35% damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:724,name:"Enduring Light",lines:["+50% well duration","60% longer well interval"],factors:Modifiers{duration:1.5,cooldown:1.6,..NEUTRAL}},
 Profile{spell:403,name:"Forked Lightning",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:403,name:"Restless Storm",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:421,name:"Far-Reaching Storm",lines:["+40% jump range","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:421,name:"Overcharged Chain",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:8042,name:"Shattering Earth",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:8042,name:"Crushing Earth",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:8050,name:"Spreading Embers",lines:["+2 targets","-30% periodic damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:8050,name:"Searing Brand",lines:["+45% periodic damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:8056,name:"Lingering Frost",lines:["+50% slow duration","-30% damage"],factors:Modifiers{effect:1.5,damage:0.7,..NEUTRAL}},
 Profile{spell:8056,name:"Shattering Frost",lines:["+40% damage","-40% slow duration"],factors:Modifiers{damage:1.4,effect:0.6,..NEUTRAL}},
 Profile{spell:324,name:"Overcharged Shield",lines:["+60% retaliation damage","70% longer cooldown"],factors:Modifiers{damage:1.6,cooldown:1.7,..NEUTRAL}},
 Profile{spell:324,name:"Volatile Shield",lines:["+40% retaliation damage","-50% duration"],factors:Modifiers{damage:1.4,duration:0.5,..NEUTRAL}},
 Profile{spell:8190,name:"Spreading Magma",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:8190,name:"Roaring Magma",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:3599,name:"Forked Embers",lines:["+1 target per attack","-35% damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:3599,name:"Blazing Embers",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:1535,name:"Expanding Nova",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:1535,name:"Intense Nova",lines:["+50% damage","-30% radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:331,name:"Deep Waters",lines:["+40% healing","50% longer cooldown"],factors:Modifiers{healing:1.4,cooldown:1.5,..NEUTRAL}},
 Profile{spell:331,name:"Swift Waters",lines:["30% shorter cooldown","-40% healing"],factors:Modifiers{cooldown:0.7,healing:0.6,..NEUTRAL}},
 Profile{spell:16038,name:"Spreading Pyre",lines:["+40% explosion radius","-25% explosion damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:16038,name:"Devouring Pyre",lines:["+50% explosion damage","-30% explosion radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:133,name:"Forked Flame",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:133,name:"Restless Fire",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:116,name:"Lingering Chill",lines:["+50% slow duration","-30% damage"],factors:Modifiers{effect:1.5,damage:0.7,..NEUTRAL}},
 Profile{spell:116,name:"Shattering Ice",lines:["+40% damage","-40% slow duration"],factors:Modifiers{damage:1.4,effect:0.6,..NEUTRAL}},
 Profile{spell:7269,name:"Arcane Barrage",lines:["+2 missiles","-30% damage per missile"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:7269,name:"Rapid Channeling",lines:["30% shorter cooldown","-25% damage per missile"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:122,name:"Deep Freeze",lines:["+50% root duration","-35% damage"],factors:Modifiers{effect:1.5,damage:0.65,..NEUTRAL}},
 Profile{spell:122,name:"Spreading Frost",lines:["+40% radius","-30% root duration"],factors:Modifiers{radius:1.4,effect:0.7,..NEUTRAL}},
 Profile{spell:1449,name:"Expanding Arcana",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:1449,name:"Dense Arcana",lines:["+50% damage","-30% radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:10,name:"Wide Blizzard",lines:["+40% radius","-25% periodic damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:10,name:"Bitter Blizzard",lines:["+40% periodic damage","-30% radius"],factors:Modifiers{damage:1.4,radius:0.7,..NEUTRAL}},
 Profile{spell:2120,name:"Spreading Inferno",lines:["+40% radius","-25% periodic damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:2120,name:"Roaring Inferno",lines:["+50% periodic damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:2136,name:"Scattered Sparks",lines:["+2 projectiles","-35% damage per projectile"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:2136,name:"Overcharged Spark",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:120,name:"Lingering Winter",lines:["+50% slow duration","-30% damage"],factors:Modifiers{effect:1.5,damage:0.7,..NEUTRAL}},
 Profile{spell:120,name:"Shattering Winter",lines:["+40% damage","-40% slow duration"],factors:Modifiers{damage:1.4,effect:0.6,..NEUTRAL}},
 Profile{spell:11113,name:"Expanding Wave",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:11113,name:"Lingering Shockwave",lines:["+50% slow duration","50% longer cooldown"],factors:Modifiers{effect:1.5,cooldown:1.5,..NEUTRAL}},
 Profile{spell:11366,name:"Twin Pyres",lines:["+1 target","-35% damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:11366,name:"Heavy Pyre",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:11426,name:"Deep Barrier",lines:["+40% absorption","60% longer cooldown"],factors:Modifiers{effect:1.4,cooldown:1.6,..NEUTRAL}},
 Profile{spell:11426,name:"Fleeting Barrier",lines:["30% shorter cooldown","-40% absorption"],factors:Modifiers{cooldown:0.7,effect:0.6,..NEUTRAL}},
 Profile{spell:118,name:"Flock Master",lines:["+2 maximum active sheep","-35% explosion damage"],factors:Modifiers{damage:0.65,..NEUTRAL}},
 Profile{spell:118,name:"Restless Flock",lines:["30% shorter transformation interval","-40% explosion damage"],factors:Modifiers{proc_rate:1.4285715,damage:0.6,..NEUTRAL}},
 Profile{spell:686,name:"Forked Shadows",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:686,name:"Restless Shadows",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:172,name:"Spreading Decay",lines:["+2 targets","-30% periodic damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:172,name:"Restless Decay",lines:["30% shorter cooldown","-25% periodic damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:348,name:"Spreading Immolation",lines:["+1 target","-35% periodic damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:348,name:"Searing Immolation",lines:["+50% periodic damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:980,name:"Spreading Agony",lines:["+2 targets","-30% periodic damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:980,name:"Crushing Agony",lines:["+45% periodic damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:5740,name:"Wide Inferno",lines:["+40% radius","-25% periodic damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:5740,name:"Dense Inferno",lines:["+50% periodic damage","-30% radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:6789,name:"Forked Coil",lines:["+2 targets","-35% damage"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:6789,name:"Ravenous Coil",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:5676,name:"Scattered Torment",lines:["+2 projectiles","-35% damage per projectile"],factors:Modifiers{targets:2,damage:0.65,..NEUTRAL}},
 Profile{spell:5676,name:"Intense Torment",lines:["+50% damage","70% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.7,..NEUTRAL}},
 Profile{spell:689,name:"Shared Hunger",lines:["+1 target","-35% periodic damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:689,name:"Voracious Drain",lines:["+45% periodic damage","60% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.6,..NEUTRAL}},
 Profile{spell:5782,name:"Spreading Dread",lines:["+2 targets","-35% fear duration"],factors:Modifiers{targets:2,effect:0.65,..NEUTRAL}},
 Profile{spell:5782,name:"Lingering Terror",lines:["+50% fear duration","50% longer cooldown"],factors:Modifiers{effect:1.5,cooldown:1.5,..NEUTRAL}},
 Profile{spell:6353,name:"Twin Souls",lines:["+1 triggered target","-35% triggered damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:6353,name:"Hungry Souls",lines:["30% shorter shard gain interval","-40% triggered damage"],factors:Modifiers{proc_rate:1.4285715,damage:0.6,..NEUTRAL}},
 Profile{spell:18708,name:"Unbound Pact",lines:["+40% demon damage buff strength","-35% duration"],factors:Modifiers{effect:1.4,duration:0.65,..NEUTRAL}},
 Profile{spell:18708,name:"Enduring Pact",lines:["+50% duration","60% longer cooldown"],factors:Modifiers{duration:1.5,cooldown:1.6,..NEUTRAL}},
 Profile{spell:5176,name:"Forked Wrath",lines:["+1 target","-25% damage"],factors:Modifiers{targets:1,damage:0.75,..NEUTRAL}},
 Profile{spell:5176,name:"Heavy Wrath",lines:["+45% damage","35% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.35,..NEUTRAL}},
 Profile{spell:8921,name:"Spreading Moonlight",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:8921,name:"Searing Moonlight",lines:["+40% damage","45% longer cooldown"],factors:Modifiers{damage:1.4,cooldown:1.45,..NEUTRAL}},
 Profile{spell:2912,name:"Twin Stars",lines:["+1 target","-35% damage"],factors:Modifiers{targets:1,damage:0.65,..NEUTRAL}},
 Profile{spell:2912,name:"Falling Star",lines:["+60% damage","50% longer cooldown"],factors:Modifiers{damage:1.6,cooldown:1.5,..NEUTRAL}},
 Profile{spell:16914,name:"Wide Tempest",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:16914,name:"Violent Tempest",lines:["+50% damage","-30% radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 Profile{spell:5570,name:"Spreading Swarm",lines:["+2 targets","-30% damage"],factors:Modifiers{targets:2,damage:0.7,..NEUTRAL}},
 Profile{spell:5570,name:"Ravenous Swarm",lines:["+45% damage","40% longer cooldown"],factors:Modifiers{damage:1.45,cooldown:1.4,..NEUTRAL}},
 Profile{spell:339,name:"Overgrowth",lines:["+2 targets","-35% root duration"],factors:Modifiers{targets:2,effect:0.65,..NEUTRAL}},
 Profile{spell:339,name:"Deep Roots",lines:["+50% root duration","40% longer cooldown"],factors:Modifiers{effect:1.5,cooldown:1.4,..NEUTRAL}},
 Profile{spell:774,name:"Abundant Growth",lines:["+40% healing per tick","-35% duration"],factors:Modifiers{healing:1.4,duration:0.65,..NEUTRAL}},
 Profile{spell:774,name:"Persistent Growth",lines:["+50% duration","-30% healing per tick"],factors:Modifiers{duration:1.5,healing:0.7,..NEUTRAL}},
 Profile{spell:8936,name:"Flourishing Regrowth",lines:["+50% healing","50% longer cooldown"],factors:Modifiers{healing:1.5,cooldown:1.5,..NEUTRAL}},
 Profile{spell:8936,name:"Swift Regrowth",lines:["35% shorter cooldown","-30% healing"],factors:Modifiers{cooldown:0.65,healing:0.7,..NEUTRAL}},
 Profile{spell:467,name:"Barbed Thorns",lines:["+50% retaliation damage","-35% duration"],factors:Modifiers{damage:1.5,duration:0.65,..NEUTRAL}},
 Profile{spell:467,name:"Lasting Thorns",lines:["+50% duration","-30% retaliation damage"],factors:Modifiers{duration:1.5,damage:0.7,..NEUTRAL}},
 Profile{spell:779,name:"Sweeping Claws",lines:["+40% radius","-25% damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:779,name:"Savage Claws",lines:["+50% damage","40% longer cooldown"],factors:Modifiers{damage:1.5,cooldown:1.4,..NEUTRAL}},
 Profile{spell:5221,name:"Rapid Shredding",lines:["30% shorter cooldown","-25% damage"],factors:Modifiers{cooldown:0.7,damage:0.75,..NEUTRAL}},
 Profile{spell:5221,name:"Rending Claws",lines:["+55% damage","45% longer cooldown"],factors:Modifiers{damage:1.55,cooldown:1.45,..NEUTRAL}},
 Profile{spell:770,name:"Far-Reaching Sparks",lines:["+40% arc radius","-25% arc damage"],factors:Modifiers{radius:1.4,damage:0.75,..NEUTRAL}},
 Profile{spell:770,name:"Potent Sparks",lines:["+50% arc damage","-30% arc radius"],factors:Modifiers{damage:1.5,radius:0.7,..NEUTRAL}},
 ];
pub const WARRIOR_SPELLS:[u32;11]=[845,1680,772,6343,12294,5308,1160,6673,23881,23922,2687];
pub const PALADIN_SPELLS:[u32;11]=[26573,879,24275,25912,2812,20271,853,635,642,20925,7328];
pub const HUNTER_SPELLS:[u32;11]=[75,3044,2643,1978,1510,13812,5116,19434,13797,13810,1130];
pub const ROGUE_SPELLS:[u32;11]=[1752,2098,13877,1943,408,1776,14278,13750,53,5171,14186];
pub const PRIEST_SPELLS:[u32;11]=[585,589,8092,15237,17,139,14914,15407,8122,2944,724];
pub const SHAMAN_SPELLS:[u32;11]=[403,421,8042,8050,8056,324,8190,3599,1535,331,16038];
pub const MAGE_SPELLS:[u32;13]=[133,116,7269,122,1449,10,2120,2136,120,11113,11366,11426,118];
pub const WARLOCK_SPELLS:[u32;11]=[686,172,348,980,5740,6789,5676,689,5782,6353,18708];
pub const DRUID_SPELLS:[u32;12]=[5176,8921,2912,16914,5570,339,774,8936,467,779,5221,770];
impl Glyph {
 pub const ALL:[Self;204]=[Self::SweepingCleave,Self::HeavyCleave,Self::WideVortex,Self::DenseVortex,Self::LingeringWounds,Self::RapidBleeding,Self::RollingThunder,Self::LastingThunder,Self::BroadStrike,Self::HeavyMortal,Self::SweepingExecution,Self::FinalVerdict,Self::LingeringDread,Self::PiercingDread,Self::LastingRally,Self::FierceRally,Self::HungeringStrike,Self::FrenziedThirst,Self::BatteringShield,Self::CrushingShield,Self::SpreadingRage,Self::FocusedRage,Self::HallowedGround,Self::LingeringConsecration,Self::ForkedExorcism,Self::SolemnExorcism,Self::SwiftWrath,Self::WeightOfWrath,Self::RadiantShock,Self::SearingShock,Self::EncirclingLight,Self::ZealousWrath,Self::SharedJudgement,Self::Condemnation,Self::LastingJustice,Self::SwiftJustice,Self::DeepPrayer,Self::QuickPrayer,Self::EnduringSanctuary,Self::FleetingSanctuary,Self::BarbedAegis,Self::LastingAegis,Self::SpreadingRedemption,Self::BrilliantRedemption,Self::ScatteredArrows,Self::HeavyArrows,Self::ArcaneSplinters,Self::ArcaneOverload,Self::Broadside,Self::RapidSalvo,Self::LingeringVenom,Self::VirulentSpread,Self::BlanketingVolley,Self::SustainedVolley,Self::WideDetonation,Self::VolatileCharge,Self::LingeringConcussion,Self::ScatteredConcussion,Self::SplitAim,Self::PatientAim,Self::SpreadingFlames,Self::EnduringEmbers,Self::WideFrost,Self::LingeringFrost,Self::FarReachingMark,Self::PotentMark,Self::SweepingBlades,Self::HeavyCuts,Self::SpreadingExecution,Self::DeepExecution,Self::WhirlingBlades,Self::ConcentratedFlurry,Self::LingeringRupture,Self::SpreadingWounds,Self::LastingSuppression,Self::SwiftSuppression,Self::SweepingGouge,Self::LingeringDaze,Self::SpectralGuard,Self::RecklessStrike,Self::EnduringRush,Self::IntenseRush,Self::ForkedBlades,Self::PatientStrike,Self::LastingEdge,Self::KeenEdge,Self::SharedFate,Self::CruelFate,Self::ScatteredFaith,Self::SwiftPrayer,Self::LingeringPain,Self::SpreadingAgony,Self::SplinteredMind,Self::CrushingThought,Self::RadiantNova,Self::MendingNova,Self::DeepShelter,Self::FleetingShelter,Self::DeepRenewal,Self::SwiftRenewal,Self::SearingFaith,Self::RestlessFlame,Self::CrushingTorment,Self::RelentlessTorment,Self::EchoingTerror,Self::SpreadingTerror,Self::SpreadingPlague,Self::RavenousPlague,Self::RadiantWell,Self::EnduringLight,Self::ForkedLightning,Self::RestlessStorm,Self::FarReachingStorm,Self::OverchargedChain,Self::ShatteringEarth,Self::CrushingEarth,Self::SpreadingEmbers,Self::SearingBrand,Self::LingeringShock,Self::ShatteringFrost,Self::OverchargedShield,Self::VolatileShield,Self::SpreadingMagma,Self::RoaringMagma,Self::ForkedEmbers,Self::BlazingEmbers,Self::ExpandingNova,Self::IntenseNova,Self::DeepWaters,Self::SwiftWaters,Self::SpreadingPyre,Self::DevouringPyre,Self::ForkedFlame,Self::RestlessFire,Self::LingeringChill,Self::ShatteringIce,Self::ArcaneBarrage,Self::RapidChanneling,Self::DeepFreeze,Self::SpreadingFrost,Self::ExpandingArcana,Self::DenseArcana,Self::WideBlizzard,Self::BitterBlizzard,Self::SpreadingInferno,Self::RoaringInferno,Self::ScatteredSparks,Self::OverchargedSpark,Self::LingeringWinter,Self::ShatteringWinter,Self::ExpandingWave,Self::LingeringShockwave,Self::TwinPyres,Self::HeavyPyre,Self::DeepBarrier,Self::FleetingBarrier,Self::FlockMaster,Self::RestlessFlock,Self::ForkedShadows,Self::RestlessShadows,Self::SpreadingDecay,Self::RestlessDecay,Self::SpreadingImmolation,Self::SearingImmolation,Self::SpreadingCurse,Self::CrushingAgony,Self::WideInferno,Self::DenseInferno,Self::ForkedCoil,Self::RavenousCoil,Self::ScatteredTorment,Self::IntenseTorment,Self::SharedHunger,Self::VoraciousDrain,Self::SpreadingDread,Self::LingeringTerror,Self::TwinSouls,Self::HungrySouls,Self::UnboundPact,Self::EnduringPact,Self::ForkedWrath,Self::HeavyWrath,Self::SpreadingMoonlight,Self::SearingMoonlight,Self::TwinStars,Self::FallingStar,Self::WideTempest,Self::ViolentTempest,Self::SpreadingSwarm,Self::RavenousSwarm,Self::Overgrowth,Self::DeepRoots,Self::AbundantGrowth,Self::PersistentGrowth,Self::FlourishingRegrowth,Self::SwiftRegrowth,Self::BarbedThorns,Self::LastingThorns,Self::SweepingClaws,Self::SavageClaws,Self::RapidShredding,Self::RendingClaws,Self::FarReachingSparks,Self::PotentSparks];
 pub fn spell(self)->u32{PROFILES[self as usize].spell}
 pub fn name(self)->&'static str{PROFILES[self as usize].name}
 pub fn lines(self)->[&'static str;2]{PROFILES[self as usize].lines}
 pub fn sheep_limit(self)->u32{if self==Self::FlockMaster{5}else{3}}
 pub fn damage_factor(self)->f32{PROFILES[self as usize].factors.damage}
 pub fn effect_factor(self)->f32{PROFILES[self as usize].factors.effect}
 pub fn healing_factor(self)->f32{PROFILES[self as usize].factors.healing}
 pub fn text(self)->String{self.lines().join("; ")}
 pub fn pair(spell:u32)->Option<[Self;2]>{Self::ALL.chunks_exact(2).find(|p|p[0].spell()==spell).map(|p|[p[0],p[1]])}
 /// Always transform a fresh native result, including its flat maximum-HP damage term.
 pub fn apply(self,mut m:Modifiers)->Modifiers{
 let f=PROFILES[self as usize].factors;
 m.damage*=f.damage;m.health_damage*=f.damage;m.cooldown*=f.cooldown;
 m.targets=m.targets.saturating_add(f.targets);m.radius*=f.radius;m.duration*=f.duration;m.effect*=f.effect;m.healing*=f.healing;m.proc_rate*=f.proc_rate;m
 }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Offer {Keep,Apply(Glyph),Remove}
impl Offer {
 pub fn result(self,previous:Option<Glyph>)->Option<Glyph>{match self {Self::Keep=>previous,Self::Apply(g)=>Some(g),Self::Remove=>None}}
}
/// Stable per native card: 80% ordinary, 10% each specialization. Luck has no influence.
/// Native rerolls may produce a different descriptor; repeated rendering cannot reroll a glyph.
pub fn offer(spell:u32,rank:u32,rarity:u64,upgrades:&[u8])->Option<Glyph> {
 match choice(spell,rank,rarity,upgrades,None){Offer::Apply(g)=>Some(g),_=>None}
}
/// At resulting rank 5+: bare spells get 80% keep / 10% each glyph.
/// Active spells get 70% keep / 20% removal / 10% the other glyph.
pub fn choice(spell:u32,rank:u32,rarity:u64,upgrades:&[u8],active:Option<Glyph>)->Offer {
 if rank<1 || rank>254 || upgrades.is_empty() || upgrades.len()>16 || rank.checked_add(upgrades.len() as u32).is_none_or(|after|after<5) {return Offer::Keep}
 let Some(pair)=Glyph::pair(spell)else{return Offer::Keep};
 let mut hash=0xcbf29ce484222325u64;
 for byte in spell.to_le_bytes().into_iter().chain(rank.to_le_bytes()).chain(rarity.to_le_bytes()).chain(upgrades.iter().copied()) {hash=(hash^u64::from(byte)).wrapping_mul(0x100000001b3);}
 // Avalanche before selecting low bits (upgrade byte sequences are strongly correlated).
 hash^=hash>>30;hash=hash.wrapping_mul(0xbf58476d1ce4e5b9);hash^=hash>>27;hash=hash.wrapping_mul(0x94d049bb133111eb);hash^=hash>>31;
 match (active,hash%10) {(Some(old),0)=>Offer::Apply(if old==pair[0]{pair[1]}else{pair[0]}),(Some(_),1|2)=>Offer::Remove,(None,0)=>Offer::Apply(pair[0]),(None,1)=>Offer::Apply(pair[1]),_=>Offer::Keep}
}
#[derive(Default)]
pub struct Glyphs {active:BTreeMap<(u32,u64,u32),Glyph>}
impl Glyphs {
 pub fn has_spell(&self,spell:u32)->bool{self.active.values().any(|g|g.spell()==spell)}
 pub fn get(&self,slot:u32,guid:u64,spell:u32)->Option<Glyph>{self.active.get(&(slot,guid,spell)).copied()}
 pub fn accept(&mut self,slot:u32,guid:u64,spell:u32,before:u32,after:u32,expected:u32,offer:Offer)->bool {
 if guid==0 || after!=expected || after<=before || after<5 {return false}
 match offer {
 Offer::Apply(g) if g.spell()==spell=>{self.active.insert((slot,guid,spell),g);true},
 Offer::Remove=>self.active.remove(&(slot,guid,spell)).is_some(),
 _=>false
 }
 }
 pub fn retain(&mut self,actors:&[(u32,u64)]){self.active.retain(|(s,g,_),_|actors.contains(&(*s,*g)));}
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn upgrade_offers_are_stable_bounded_and_spell_specific(){for s in [845,1680,772] {let mut seen=[false;3];for rank in 1..100 {for rarity in 0..5 {let a=offer(s,rank,rarity,&[0,2]);assert_eq!(a,offer(s,rank,rarity,&[0,2]));if let Some(g)=a {assert_eq!(g.spell(),s);seen[if matches!(g,Glyph::SweepingCleave|Glyph::WideVortex|Glyph::LingeringWounds){0}else{1}]=true;}else{seen[2]=true}}}assert_eq!(seen,[true;3]);}assert_eq!(offer(845,0,0,&[0]),None);assert_eq!(offer(1,1,0,&[0]),None);}
 #[test] fn replacement_keep_rejection_and_identity(){let mut s=Glyphs::default();assert!(s.accept(0,10,845,4,5,5,Offer::Apply(Glyph::SweepingCleave)));assert!(!s.accept(0,10,845,5,6,6,Offer::Keep));assert_eq!(s.get(0,10,845),Some(Glyph::SweepingCleave));assert!(!s.accept(0,10,845,6,6,7,Offer::Apply(Glyph::HeavyCleave)));assert_eq!(s.get(1,10,845),None);assert_eq!(s.get(0,11,845),None);assert!(s.accept(0,10,845,6,7,7,Offer::Apply(Glyph::HeavyCleave)));s.retain(&[(0,11)]);assert_eq!(s.get(0,10,845),None);}
 #[test] fn removal_is_explicit_rank_checked_and_owner_local(){
 let mut s=Glyphs::default();assert!(s.accept(0,10,845,4,5,5,Offer::Apply(Glyph::HeavyCleave)));
 assert!(!s.accept(0,10,845,5,5,6,Offer::Remove));assert_eq!(s.get(0,10,845),Some(Glyph::HeavyCleave));
 assert!(!s.accept(1,10,845,5,6,6,Offer::Remove));
 assert!(s.accept(0,10,845,5,6,6,Offer::Remove));assert_eq!(s.get(0,10,845),None);
 assert!(!s.accept(0,10,845,5,6,6,Offer::Remove));
 assert!(s.accept(0,10,845,6,7,7,Offer::Apply(Glyph::SweepingCleave)));
 }
 #[test] fn rank_gate_and_active_removal_distribution(){
 for before in 0..4{for rarity in 0..5{assert_eq!(choice(845,before,rarity,&[0],None),Offer::Keep);}}
 assert_eq!(choice(845,1,0,&[],None),Offer::Keep);
 let mut bare=[0usize;3];let mut active=[0usize;3];
 for rank in 4..=100{for rarity in 0..5{for upgrade in 0..16{
  match choice(845,rank,rarity,&[upgrade],None){Offer::Keep=>bare[0]+=1,Offer::Apply(g)=>bare[if g==Glyph::SweepingCleave{1}else{2}]+=1,_=>panic!("bare spell cannot remove")}
  match choice(845,rank,rarity,&[upgrade],Some(Glyph::SweepingCleave)){Offer::Keep=>active[0]+=1,Offer::Remove=>active[1]+=1,Offer::Apply(g)=>{assert_eq!(g,Glyph::HeavyCleave);active[2]+=1;}}
 }}}
 let total=active.iter().sum::<usize>() as f32;for(count,wanted)in active.into_iter().zip([0.70,0.20,0.10]){assert!((count as f32/total-wanted).abs()<0.025);}for(count,wanted)in bare.into_iter().zip([0.80,0.10,0.10]){assert!((count as f32/total-wanted).abs()<0.025);}
 let mut s=Glyphs::default();assert!(!s.accept(0,10,845,1,2,2,Offer::Apply(Glyph::HeavyCleave)));assert!(s.accept(0,10,845,4,5,5,Offer::Apply(Glyph::HeavyCleave)));
 }
 #[test] fn full_warrior_registry_has_two_tradeoffs_per_spell_and_no_target_reduction(){
 let mut names=std::collections::BTreeSet::new();
 for spell in WARRIOR_SPELLS {let pair=Glyph::pair(spell).unwrap();assert_ne!(pair[0],pair[1]);
 for glyph in pair {
 assert_eq!(glyph.spell(),spell);assert!(names.insert(glyph.name()));
 let m=glyph.apply(Modifiers{targets:4,health_damage:0.1,..Default::default()});
 assert!(m.targets>=4);assert!([m.damage,m.cooldown,m.radius,m.duration,m.health_damage,m.effect].iter().all(|f|f.is_finite()&&*f>0.));
 let f=PROFILES[glyph as usize].factors;
 assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.duration>1.||f.effect>1.);
 assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.||f.effect<1.);
 let mut state=Glyphs::default();assert!(state.accept(0,10,spell,4,5,5,Offer::Apply(glyph)));assert!(!state.accept(0,10,spell,5,6,6,Offer::Keep));assert_eq!(state.get(0,10,spell),Some(glyph));assert!(state.accept(0,10,spell,6,7,7,Offer::Remove));assert_eq!(state.get(0,10,spell),None);
 let mut offered=std::collections::BTreeSet::new();for rank in 1..100{for rarity in 0..5 {if let Offer::Apply(g)=choice(spell,rank,rarity,&[0,2],None){offered.insert(g as usize);}}}assert!(pair.iter().all(|g|offered.contains(&(*g as usize))));
 }}assert_eq!(names.len(),22);assert_eq!(Glyph::pair(999999),None);
 }
 #[test] fn buff_control_and_hit_profiles_scale_independent_native_channels(){
 let m=Modifiers{damage:2.,cooldown:0.5,targets:4,radius:1.2,duration:1.3,health_damage:0.1,effect:1.75,healing:1.,proc_rate:1.};
 let fierce=Glyph::FierceRally.apply(m);assert!((fierce.effect-2.45).abs()<0.00001);assert!((fierce.duration-0.845).abs()<0.00001);assert_eq!(fierce.damage,m.damage);
 let shield=Glyph::CrushingShield.apply(m);assert!((shield.effect-1.05).abs()<0.00001);assert!((shield.health_damage-0.14).abs()<0.00001);assert_eq!(shield.targets,4);
 assert_eq!(Glyph::LastingThunder.apply(m).effect,m.effect);
 assert_eq!(Glyph::FrenziedThirst.apply(m).effect,m.effect);
 }
 #[test] fn paladin_profiles_are_complete_and_healing_is_independent_of_damage(){
 let mut names=std::collections::BTreeSet::new();for spell in PALADIN_SPELLS{let pair=Glyph::pair(spell).unwrap();for glyph in pair{assert!(names.insert(glyph.name()));assert_eq!(glyph.spell(),spell);let m=glyph.apply(Modifiers{targets:3,..Default::default()});assert!(m.targets>=3);let f=PROFILES[glyph as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.duration>1.||f.effect>1.||f.healing>1.);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.||f.effect<1.||f.healing<1.);let mut state=Glyphs::default();assert!(state.accept(1,20,spell,4,5,5,Offer::Apply(glyph)));assert_eq!(state.get(0,20,spell),None);assert!(state.accept(1,20,spell,5,6,6,Offer::Remove));assert_eq!(state.get(1,20,spell),None);}}assert_eq!(names.len(),22);
 let shock=Glyph::RadiantShock.apply(Modifiers::default());assert_eq!(shock.healing,1.5);assert_eq!(shock.damage,0.7);
 let prayer=Glyph::DeepPrayer.apply(Modifiers::default());assert_eq!(prayer.damage,1.);assert_eq!(prayer.healing,1.4);
 }
 #[test] fn hunter_profiles_keep_target_identity_and_have_tradeoffs(){
 let mut names=std::collections::BTreeSet::new();for spell in HUNTER_SPELLS{let pair=Glyph::pair(spell).unwrap();for glyph in pair{assert!(names.insert(glyph.name()));let m=glyph.apply(Modifiers{targets:3,..Default::default()});assert!(m.targets>=3);let f=PROFILES[glyph as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.duration>1.);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.);let mut state=Glyphs::default();assert!(state.accept(0,10,spell,4,5,5,Offer::Apply(glyph)));assert_eq!(state.get(1,10,spell),None);assert!(state.accept(0,10,spell,5,6,6,Offer::Keep)==false);assert!(state.accept(0,10,spell,6,7,7,Offer::Remove));assert_eq!(state.get(0,10,spell),None);}}assert_eq!(names.len(),22);
 assert_eq!(Glyph::LingeringConcussion.apply(Modifiers::default()).effect,1.);assert_eq!(Glyph::WideFrost.apply(Modifiers::default()).effect,1.);
 }
 #[test] fn rogue_kit_has_tradeoffs_and_seal_fate_is_independent_of_eviscerate(){
 let mut names=std::collections::BTreeSet::new();for spell in ROGUE_SPELLS{for g in Glyph::pair(spell).unwrap(){assert!(names.insert(g.name()));let f=PROFILES[g as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.duration>1.||f.effect>1.);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.||f.effect<1.||f.proc_rate<1.);assert!(g.apply(Modifiers{targets:3,..Default::default()}).targets>=3);}}
 assert_eq!(names.len(),22);let mut state=Glyphs::default();assert!(state.accept(0,10,2098,4,5,5,Offer::Apply(Glyph::DeepExecution)));assert!(state.accept(0,10,14186,4,5,5,Offer::Apply(Glyph::SharedFate)));assert_eq!(state.get(0,10,2098),Some(Glyph::DeepExecution));assert_eq!(state.get(1,10,14186),None);assert!(state.accept(0,10,14186,5,6,6,Offer::Remove));assert_eq!(state.get(0,10,14186),None);assert_eq!(state.get(0,10,2098),Some(Glyph::DeepExecution));
 let cruel=Glyph::CruelFate.apply(Modifiers::default());assert_eq!(cruel.proc_rate,0.7);assert_eq!(cruel.cooldown,1.);assert_eq!(cruel.damage,1.5);
 let spectral=Glyph::SpectralGuard.apply(Modifiers::default());assert_eq!((spectral.damage,spectral.effect),(0.7,1.4));assert_eq!(Glyph::KeenEdge.apply(Modifiers::default()).damage,1.);
 }
 #[test] fn priest_profiles_have_independent_healing_and_preserve_multitarget_identity(){
 let mut names=std::collections::BTreeSet::new();for spell in PRIEST_SPELLS{for g in Glyph::pair(spell).unwrap(){assert!(names.insert(g.name()));let f=PROFILES[g as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.duration>1.||f.effect>1.||f.healing>1.);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.||f.effect<1.||f.healing<1.);assert!(g.apply(Modifiers{targets:3,..Default::default()}).targets>=3);let mut state=Glyphs::default();assert!(state.accept(0,10,spell,4,5,5,Offer::Apply(g)));assert_eq!(state.get(1,10,spell),None);assert!(state.accept(0,10,spell,5,6,6,Offer::Remove));assert_eq!(state.get(0,10,spell),None);}}assert_eq!(names.len(),22);
 let nova=Glyph::MendingNova.apply(Modifiers::default());assert_eq!((nova.damage,nova.healing),(0.7,1.5));assert_eq!(Glyph::RadiantNova.apply(Modifiers::default()).healing,1.);assert_eq!(Glyph::DeepRenewal.apply(Modifiers::default()).duration,1.);assert_eq!(Glyph::DeepShelter.apply(Modifiers::default()).effect,1.4);
 }
 #[test] fn shaman_profiles_have_tradeoffs_and_keep_source_identity(){
 let mut names=std::collections::BTreeSet::new();for spell in SHAMAN_SPELLS{for g in Glyph::pair(spell).unwrap(){assert!(names.insert(g.name()));let f=PROFILES[g as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.effect>1.||f.healing>1.);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.||f.effect<1.||f.healing<1.);assert!(g.apply(Modifiers{targets:3,..Default::default()}).targets>=3);let mut state=Glyphs::default();assert!(state.accept(0,10,spell,4,5,5,Offer::Apply(g)));assert_eq!(state.get(1,10,spell),None);assert!(state.accept(0,10,spell,5,6,6,Offer::Remove));assert_eq!(state.get(0,10,spell),None);}}assert_eq!(names.len(),22);
 assert_eq!(Glyph::DeepWaters.apply(Modifiers::default()).healing,1.4);assert_eq!(Glyph::LingeringShock.apply(Modifiers::default()).duration,1.);assert_eq!(Glyph::SpreadingPyre.apply(Modifiers::default()).cooldown,1.);assert_eq!(Glyph::VolatileShield.apply(Modifiers::default()).effect,1.);
 }
 #[test] fn mage_sheep_capacity_and_tradeoffs_are_owner_local(){
 let mut names=std::collections::BTreeSet::new();for sp in MAGE_SPELLS{for g in Glyph::pair(sp).unwrap(){assert!(names.insert(g.name()));let f=PROFILES[g as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.effect>1.||f.proc_rate>1.||g.sheep_limit()>3);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.effect<1.);assert!(g.apply(Modifiers{targets:3,..Default::default()}).targets>=3);}}
 assert_eq!(names.len(),26);assert_eq!(Glyph::FlockMaster.sheep_limit(),5);assert_eq!(Glyph::RestlessFlock.sheep_limit(),3);
 let mut s=Glyphs::default();assert!(s.accept(0,11,118,4,5,5,Offer::Apply(Glyph::FlockMaster)));assert_eq!(s.get(1,12,118),None);assert!(s.accept(0,11,118,5,6,6,Offer::Remove));assert_eq!(s.get(0,11,118),None);
 }
 #[test] fn warlock_profiles_keep_native_leech_and_source_local_shards(){
 let mut names=std::collections::BTreeSet::new();for sp in WARLOCK_SPELLS{for g in Glyph::pair(sp).unwrap(){assert!(names.insert(g.name()));let f=PROFILES[g as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.duration>1.||f.effect>1.||f.proc_rate>1.);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.||f.effect<1.);assert!(g.apply(Modifiers{targets:3,..Default::default()}).targets>=3);let mut s=Glyphs::default();assert!(s.accept(0,11,sp,4,5,5,Offer::Apply(g)));assert_eq!(s.get(1,12,sp),None);assert!(s.accept(0,11,sp,5,6,6,Offer::Remove));assert_eq!(s.get(0,11,sp),None);}}
 assert_eq!(names.len(),22);assert_eq!(Glyph::pair(1949),None);assert_eq!(Glyph::HungrySouls.apply(Modifiers::default()).cooldown,1.);assert!(Glyph::HungrySouls.apply(Modifiers::default()).proc_rate>1.4);for g in [Glyph::ForkedCoil,Glyph::RavenousCoil,Glyph::SharedHunger,Glyph::VoraciousDrain]{assert_eq!(g.apply(Modifiers::default()).healing,1.);assert_eq!(g.apply(Modifiers::default()).effect,1.);}
 }
 #[test] fn final_factors_cover_flat_hp_and_preserve_multitarget_identity(){let m=Modifiers{damage:2.,health_damage:0.1,targets:4,radius:2.,..Default::default()};let n=Glyph::SweepingCleave.apply(m);assert_eq!(n.targets,6);assert_eq!(n.damage,1.5);assert!((n.health_damage-0.075).abs()<0.00001);assert_eq!(Glyph::DenseVortex.apply(m).targets,4);assert_eq!(Glyph::WideVortex.apply(m).radius,2.8);assert_eq!(Glyph::HeavyCleave.apply(m).cooldown,1.5);assert_eq!(Glyph::SweepingCleave.apply(m),n);}
 #[test] fn bleeds_trade_intensity_against_duration(){let a=Glyph::LingeringWounds.apply(Modifiers::default());let b=Glyph::RapidBleeding.apply(Modifiers::default());assert_eq!(a.damage*a.duration,1.125);assert_eq!(b.damage*b.duration,0.75);assert_eq!(a.targets,b.targets);}
 #[test] fn druid_profiles_have_tradeoffs_and_owner_local_removal(){
 let mut names=std::collections::BTreeSet::new();for sp in DRUID_SPELLS{for g in Glyph::pair(sp).unwrap(){assert!(names.insert(g.name()));let f=PROFILES[g as usize].factors;assert!(f.damage>1.||f.cooldown<1.||f.targets>0||f.radius>1.||f.duration>1.||f.effect>1.||f.healing>1.);assert!(f.damage<1.||f.cooldown>1.||f.radius<1.||f.duration<1.||f.effect<1.||f.healing<1.);assert!(g.apply(Modifiers{targets:3,..Default::default()}).targets>=3);let mut s=Glyphs::default();assert!(s.accept(0,11,sp,4,5,5,Offer::Apply(g)));assert_eq!(s.get(1,12,sp),None);assert!(s.accept(0,11,sp,5,6,6,Offer::Remove));assert_eq!(s.get(0,11,sp),None);}}
 assert_eq!(names.len(),24);assert_eq!(Glyph::AbundantGrowth.apply(Modifiers::default()).healing,1.4);assert_eq!(Glyph::DeepRoots.apply(Modifiers::default()).duration,1.);assert_eq!(Glyph::PotentSparks.apply(Modifiers::default()).effect,1.);
 }

}
