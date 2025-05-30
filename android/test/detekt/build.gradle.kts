plugins {
    kotlin("jvm")
}

dependencies {
    compileOnly("io.gitlab.arturbosch.detekt:detekt-api:1.23.8")
    testImplementation("io.gitlab.arturbosch.detekt:detekt-test:1.23.8")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-api:5.7.0")
    testImplementation("org.junit.jupiter:junit-jupiter-engine:5.7.0")
}

tasks.withType<Test> {
    useJUnitPlatform()
}
