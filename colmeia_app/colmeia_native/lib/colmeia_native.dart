import 'dart:ffi';
import 'dart:io';
import 'dart:async';

import 'package:flutter/services.dart';

final DynamicLibrary nativeAddLib = Platform.isAndroid
    ? DynamicLibrary.open("libcolmeia_native.so")
    : DynamicLibrary.process();

class ColmeiaNative {
  static const MethodChannel _channel = const MethodChannel('colmeia_native');

  static Future<String> get platformVersion async {
    final String version = await _channel.invokeMethod('getPlatformVersion');
    return version;
  }
}
