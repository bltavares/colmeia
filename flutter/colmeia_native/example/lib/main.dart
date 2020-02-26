import 'package:flutter/material.dart';
import 'dart:async';
import 'dart:isolate';

import 'package:colmeia_native/colmeia_native.dart';

void main() => runApp(MyApp());

class MyApp extends StatefulWidget {
  @override
  _MyAppState createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  String _platformVersion = 'Unknown';
  int server = 0;

  // Platform messages are asynchronous, so we initialize in an async method.

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(
          title: const Text('Plugin example app'),
        ),
        body: Center(
          child: Text('Running on: $_platformVersion\n'),
        ),
        floatingActionButton: FloatingActionButton(
            backgroundColor: server == 0 ? Colors.blue : Colors.red,
            mini: server > 0,
            onPressed: () {
              if (server == 0) {
                Isolate.spawn(sync, {});
                setState(() {
                  server = 1;
                });
              }
            }),
      ),
    );
  }
}
