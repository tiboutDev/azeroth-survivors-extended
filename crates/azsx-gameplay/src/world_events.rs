//! Shared event placement policy, independent of player position and client layout.
#[derive(Clone,Copy,Debug)]pub struct Area{pub center:[f32;2],pub radius:f32}
impl Area{
 pub fn sample(self,margin:f32,radial:u32,angle:u32)->Option<[f32;2]>{
  if !self.radius.is_finite()||!margin.is_finite()||margin<0.||self.radius<=margin||self.center.iter().any(|v|!v.is_finite()){return None}
  let r=(self.radius-margin)*((radial as f64+0.5)/4294967296.).sqrt()as f32;
  let a=((angle as f64+0.5)/4294967296.*std::f64::consts::TAU)as f32;
  Some([self.center[0]+r*a.cos(),self.center[1]+r*a.sin()])
 }
 pub fn contains(self,point:[f32;2],margin:f32)->bool{point.iter().chain(self.center.iter()).all(|v|v.is_finite())&&margin.is_finite()&&margin>=0.&&self.radius.is_finite()&&self.radius>=margin&&(point[0]-self.center[0]).hypot(point[1]-self.center[1])<=self.radius-margin}
}
pub fn separated(point:[f32;2],radius:f32,occupied:&[([f32;3],f32)])->bool{occupied.iter().all(|(p,r)|(point[0]-p[0]).hypot(point[1]-p[1])>=radius+r+3.)}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn sample_covers_area_without_a_player_input_and_respects_footprint(){let a=Area{center:[10.,20.],radius:100.};let near=a.sample(9.,0,0).unwrap();let far=a.sample(9.,u32::MAX,0).unwrap();assert!((near[0]-10.).abs()<0.01);assert!((far[0]-101.).abs()<0.01);for n in 0..1000{let p=a.sample(9.,n*4294967,n*3943621).unwrap();assert!(a.contains(p,8.99));}assert!(!separated([10.,20.],2.,&[([12.,20.,0.],8.)]));}
 #[test]fn invalid_geometry_is_rejected(){assert!(Area{center:[0.;2],radius:8.}.sample(9.,0,0).is_none());assert!(Area{center:[f32::NAN,0.],radius:20.}.sample(2.,0,0).is_none());}
}

/// Native terrain observations only; inclination is a preference, never a veto.
#[derive(Clone,Copy,Debug,PartialEq)]pub struct GroundSite{pub center:[f32;3],pub low:f32,pub high:f32}
impl GroundSite{
 pub fn variation(self)->Option<f32>{let span=self.high-self.low;(self.center.iter().all(|v|v.is_finite())&&self.low.is_finite()&&self.high.is_finite()&&span.is_finite()&&self.low<=self.center[2]&&self.center[2]<=self.high).then_some(span)}
 pub fn position(self,preferred:f32)->[f32;3]{let mut p=self.center;p[2]=if self.high-self.low<=preferred{self.high}else{self.center[2]};p[2]+=0.025;p}
}
pub fn choose_site(sites:&[GroundSite],preferred:f32)->Option<GroundSite>{
 if !preferred.is_finite()||preferred<0.{return None}
 let mut best=None;let mut best_span=f32::INFINITY;
 for site in sites{let Some(span)=site.variation()else{continue};if span<=preferred{return Some(*site)}if span<best_span{best=Some(*site);best_span=span;}}
 best
}
#[cfg(test)]mod terrain_tests{
 use super::*;
 #[test]fn flat_sites_are_preferred_but_slopes_never_prevent_placement(){
  let steep=GroundSite{center:[1.,2.,5.],low:1.,high:9.};let slope=GroundSite{center:[3.,4.,2.],low:1.,high:3.};let flat=GroundSite{center:[5.,6.,1.1],low:1.,high:1.2};
  assert_eq!(choose_site(&[steep,flat,slope],0.6),Some(flat));assert_eq!(choose_site(&[steep,slope],0.6),Some(slope));assert_eq!(choose_site(&[steep],0.6),Some(steep));
  assert!((slope.position(0.6)[2]-2.025).abs()<0.001);assert!((flat.position(0.6)[2]-1.225).abs()<0.001);assert!(choose_site(&[],0.6).is_none());
 }
 #[test]fn invalid_native_samples_do_not_become_fallbacks(){let bad=GroundSite{center:[f32::NAN,0.,0.],low:0.,high:0.};assert!(choose_site(&[bad],0.6).is_none());assert!(choose_site(&[GroundSite{center:[0.,0.,2.],low:0.,high:1.}],0.6).is_none());}
}
