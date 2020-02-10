import 'dart:ffi';
import 'dart:io';

final DynamicLibrary colmeia = Platform.isAndroid
    ? DynamicLibrary.open("libcolmeia_dat1_ffi.so")
    : DynamicLibrary.process();

final void Function() colmeiaSync = colmeia
    .lookup<NativeFunction<Void Function()>>("colmeia_dat1_sync")
    .asFunction();

void sync(_) async => colmeiaSync();
