//与使用方的App保持兼容:Kotlin 1.9编译的库,Kotlin 1.9及以上的项目都可以使用
plugins {
    id("com.android.library") version "8.2.1" apply false
    id("org.jetbrains.kotlin.android") version "1.9.0" apply false
}
