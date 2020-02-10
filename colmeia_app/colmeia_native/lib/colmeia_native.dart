import 'dart:ffi';
import 'dart:io';
import 'dart:async';

import 'package:flutter/services.dart';

final DynamicLibrary colmeia = Platform.isAndroid
    ? DynamicLibrary.open("libcolmeia_dat1_ffi.so")
    : DynamicLibrary.process();

final void Function() colmeiaSync =
    colmeia.lookup<NativeFunction<Void Function()>>("sync").asFunction();

void sync(_) async => colmeiaSync();

class ColmeiaNative {
  static const MethodChannel _channel = const MethodChannel('colmeia_native');

  static Future<String> get platformVersion async {
    final String version = await _channel.invokeMethod('getPlatformVersion');
    return version;
  }
}
