# JNA通过反射访问这些类
-dontwarn java.awt.*
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keep class cn.dairo.file.sdk.** { *; }
