// A placeholder executable, replaced before the app is ever signed.
//
// The game is not built by Xcode: `cargo build --target aarch64-apple-ios` produces the executable, and rustc
// will not emit a library from a bin target, so there is nothing here for Xcode to compile against the game's
// code. Xcode's job is the bundle and the signature -- automatic provisioning, which is the thing a phone
// actually requires and the thing no shell script reproduces. So this file exists to give the target something
// to link, and project.yml's post-compile script overwrites the result with the Rust binary.
//
// If you are reading a log that started at this `main`, that copy did not happen and the bundle is a shell with
// nothing in it. `cargo build --release --target aarch64-apple-ios -p horizon-client --no-default-features` is
// the command the build phase runs; its output goes to target/aarch64-apple-ios/release/horizon-client.
#import <Foundation/Foundation.h>

int main(int argc, const char *argv[]) {
  @autoreleasepool {
    NSLog(@"Horizon: this bundle holds the placeholder executable, not the game. The Rust binary was not copied \
over it, so the build phase that copies it did not run or did not find target/aarch64-apple-ios/release/horizon-client.");
    return 1;
  }
}
