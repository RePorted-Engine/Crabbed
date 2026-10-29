"""Run the production reload poller with a deliberately blocked pack worker."""
from pathlib import Path
import os
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class LocalPackReloadTests(unittest.TestCase):
    def test_worker_does_not_block_and_latest_selection_wins(self):
        source = (ROOT / 'crates/horizon-client/src/game/packs.rs').read_text()
        start = source.index('    pub(super) fn poll_local_pack_reload(')
        opening = source.index('{', start)
        end, depth = opening + 1, 1
        while depth:
            depth += (source[end] == '{') - (source[end] == '}')
            end += 1
        method = source[start:end].replace('pub(super)', '')
        fixture = r'''
extern crate self as log;
#[macro_export] macro_rules! warn { ($($arg:tt)*) => {} }
#[derive(Default)] struct Options { resource_error: Option<String> }
#[derive(Default)] struct Ui { options: Options }
#[derive(Default)] struct Server { reload: Option<()> }
#[derive(Default)] struct Game {
 local_pack_reload: Option<std::thread::JoinHandle<Result<u32,String>>>,
 pending_pack_reload: bool, server_packs: Server, rest_loader: Option<()>,
 world_loader: Option<()>, ui: Ui, applied: Vec<u32>, starts: usize,
}
impl Game {
 fn apply_staged_pack(&mut self, value:u32)->Result<(),String> {self.applied.push(value);Ok(())}
 fn reload_packs(&mut self) {self.starts+=1;}
 METHOD
}
fn main() {
 let mut g=Game::default();
 let (send,recv)=std::sync::mpsc::channel();
 g.local_pack_reload=Some(std::thread::spawn(move || {recv.recv().unwrap();Ok(1)}));
 g.poll_local_pack_reload(); // Must return while the worker is blocked.
 assert!(g.applied.is_empty());
 g.pending_pack_reload=true;
 send.send(()).unwrap();
 while !g.local_pack_reload.as_ref().unwrap().is_finished() {std::thread::yield_now();}
 g.poll_local_pack_reload();
 assert!(g.applied.is_empty()); assert_eq!(g.starts,1);
 for value in [Ok(2),Err("bad pack".to_string())] {
  g.local_pack_reload=Some(std::thread::spawn(move || value));
  while !g.local_pack_reload.as_ref().unwrap().is_finished() {std::thread::yield_now();}
  g.poll_local_pack_reload();
 }
 assert_eq!(g.applied,vec![2]);
 assert_eq!(g.ui.options.resource_error.as_deref(),Some("bad pack"));
 for world in [true,false] {
  g.local_pack_reload=Some(std::thread::spawn(|| Ok(3)));
  while !g.local_pack_reload.as_ref().unwrap().is_finished() {std::thread::yield_now();}
  if world {g.world_loader=Some(());} else {g.rest_loader=Some(());}
  let before=g.applied.len();
  g.poll_local_pack_reload();
  assert!(g.local_pack_reload.is_some()); assert_eq!(g.applied.len(),before);
  g.world_loader=None; g.rest_loader=None;
  g.poll_local_pack_reload();
  assert!(g.local_pack_reload.is_none()); assert_eq!(g.applied.len(),before+1);
 }
 g.pending_pack_reload=true; g.server_packs.reload=Some(());
 g.poll_local_pack_reload(); assert_eq!(g.starts,1); assert!(g.pending_pack_reload);
 g.server_packs.reload=None; g.poll_local_pack_reload(); assert_eq!(g.starts,2);
}
'''.replace('METHOD', method)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path/'test.rs').write_text(fixture)
            build = subprocess.run(['rustc','--edition=2024',str(path/'test.rs'),'-o',str(path/'test')],capture_output=True,text=True)
            self.assertEqual(build.returncode,0,build.stderr)
            subprocess.run([str(path/'test')],check=True,timeout=5)
