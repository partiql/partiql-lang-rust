plugins {
    `java-library`
    `maven-publish`
    id("me.champeau.jmh") version "0.7.2"
}

group = "org.partiql"
version = "0.14.0"

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
    withSourcesJar()
    withJavadocJar()
}

repositories {
    mavenCentral()
}

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
    testImplementation("org.assertj:assertj-core:3.24.2")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
    
    // JMH dependencies
    jmh("org.openjdk.jmh:jmh-core:1.37")
    jmh("org.openjdk.jmh:jmh-generator-annprocess:1.37")
    
    // Comparison library for benchmarks (transitively includes partiql-spi, partiql-plan, etc.)
    jmh("org.partiql:partiql-eval:1.3.3")
    jmh("org.partiql:partiql-parser:1.3.3")
    jmh("org.partiql:partiql-planner:1.3.3")
    jmh("org.jetbrains.kotlin:kotlin-stdlib:1.9.24")
}

// Determine OS-specific library name
val nativeLibName: String by lazy {
    val osName = System.getProperty("os.name").lowercase()
    when {
        osName.contains("mac") -> "libpartiql_jni.dylib"
        osName.contains("win") -> "partiql_jni.dll"
        else -> "libpartiql_jni.so"
    }
}

// Task to build Rust library
val buildRustLib = tasks.register<Exec>("buildRustLib") {
    description = "Build the Rust native library using Cargo"
    group = "build"
    
    workingDir = file("rust")
    commandLine = listOf("cargo", "build", "--release")
    
    // Outputs for up-to-date checking (builds to workspace root target/)
    outputs.file(file("../target/release/$nativeLibName"))
}

// Task to copy native library to resources
val copyNativeLib = tasks.register<Copy>("copyNativeLib") {
    description = "Copy native library to resources directory"
    group = "build"
    
    dependsOn(buildRustLib)
    
    // Workspace member builds to workspace root target/
    val targetDir = file("../target/release")
    val resourceDir = file("src/main/resources/native")
    
    from(targetDir) {
        include("*.so", "*.dylib", "*.dll")
    }
    into(resourceDir)
    
    // Handle duplicates by including them (overwrite with same file)
    duplicatesStrategy = DuplicatesStrategy.INCLUDE
    
    // Create resources directory if it doesn't exist
    doFirst {
        resourceDir.mkdirs()
    }
}

// Make processResources depend on copying native lib
tasks.named<ProcessResources>("processResources") {
    dependsOn(copyNativeLib)
    
    // Handle duplicates when processing resources
    duplicatesStrategy = DuplicatesStrategy.INCLUDE
}

// Make compileJava depend on processResources
tasks.named("compileJava") {
    dependsOn("copyNativeLib")
}

tasks.named<Jar>("sourcesJar") {
    dependsOn("copyNativeLib")
    duplicatesStrategy = DuplicatesStrategy.INCLUDE
}

// Task to clean Rust artifacts
val cleanRust = tasks.register<Exec>("cleanRust") {
    description = "Clean Rust build artifacts"
    group = "build"
    
    workingDir = file("rust")
    commandLine = listOf("cargo", "clean")
    
    // Ignore exit value in case cargo is not available
    isIgnoreExitValue = true
}

// Make clean task depend on Rust clean
tasks.named("clean") {
    dependsOn(cleanRust)
    doLast {
        // Also clean the resources directory
        delete(file("src/main/resources/native"))
    }
}

// Test configuration
tasks.test {
    useJUnitPlatform()
    
    testLogging {
        events("passed", "skipped", "failed")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
        showStandardStreams = false
    }
    
    // Ensure native library is available for tests
    dependsOn(copyNativeLib)
}

// Maven publishing configuration
publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])
            
            pom {
                name.set("PartiQL JNI")
                description.set("JNI bindings for PartiQL Rust engine")
                url.set("https://github.com/partiql/partiql-lang-rust")
                
                licenses {
                    license {
                        name.set("Apache License 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0")
                    }
                }
                
                developers {
                    developer {
                        name.set("PartiQL Team")
                        email.set("partiql-team@amazon.com")
                    }
                }
                
                scm {
                    connection.set("scm:git:git://github.com/partiql/partiql-lang-rust.git")
                    developerConnection.set("scm:git:ssh://github.com/partiql/partiql-lang-rust.git")
                    url.set("https://github.com/partiql/partiql-lang-rust")
                }
            }
        }
    }
}

// JMH configuration
val jmhIncludes = findProperty("jmhIncludes").toString()
jmh {
    if (jmhIncludes != null) {
        includes.add(jmhIncludes)
    } else {
        includes.add(".*Benchmark*")
    }
    jvmArgs.add("-Djava.library.path=src/main/resources/native")
    
    warmupIterations.set(5)
    iterations.set(10)
    fork.set(2)
    threads.set(1)
    
    resultFormat.set("JSON")
    resultsFile.set(project.file("${layout.buildDirectory.get()}/reports/jmh/results.json"))
}

// Make jmh tasks depend on native library
tasks.named("jmhCompileGeneratedClasses") {
    dependsOn("copyNativeLib")
}

tasks.named("jmh") {
    dependsOn("copyNativeLib")
}

// Task to display build info
tasks.register("buildInfo") {
    description = "Display build information"
    group = "help"
    
    doLast {
        println("PartiQL JNI Build Information")
        println("==============================")
        println("Project: ${project.name}")
        println("Version: ${project.version}")
        println("Java Version: ${java.sourceCompatibility}")
        println("OS: ${System.getProperty("os.name")}")
        println("Native Library: $nativeLibName")
        println("Build Directory: ${layout.buildDirectory.get().asFile}")
    }
}
