@Library('KOSi Pipeline Library@v3.3.2') _

import groovy.transform.Field

@Field
def linux = "rhel-7-gcc-10.2-release"
@Field
def windows = "vs-15-release"

def pipelineParams = [:]

def projectConfig = [
    projectName: 'siggen_toolkit',
    profiles: [ windows, linux ],
  	primaryNode: linux,
  	cleanOnFailure: true,
    slack: [
        channel: '#proj-siggen_toolkit-ci',
        onlyOnFail: 'true',
        ]
    ]

pipeline {
    agent none

    options {
        timeout(time: 120, unit: 'MINUTES', activity: true)
        disableConcurrentBuilds()
    }

    stages {
      	stage ('Load Pipeline Config') {
            agent { label pipelineParams.executor ?: "kosipipelineexecutor" }

            steps {
                script {
                    pipelineParams = ConanGetContext(pipelineParams, projectConfig)
                  	CleanWorkspace(pipelineParams)
                }
            }
        }

        stage('Get Vault Token') {
            agent { label pipelineParams.vault.agentLabelForNodeToAuthenticateWithVault }

            steps {
                script {
                    pipelineParams = InsertVaultToken(pipelineParams)
                  	CleanWorkspace(pipelineParams)
                }
            }
        }

        stage ('Set Conan Environment') {
          	agent { label "kosipipelineexecutor" }

            steps {
                script {
                    pipelineParams = ConanInsertAutomaticParams(pipelineParams)
                  	CleanWorkspace(pipelineParams)
                }
            }
        }

        stage ('Build') {
            agent none

            steps {
                script {
                    RunParallelPipeline(pipelineParams)
                }
            }
        }
    }

    post {
        failure {
            script {
                SendMessageToSlackViaJava(projectConfig, "Build Failed", null)
            }
        }
    }
}

def RunParallelPipeline(Map pipelineParams = [:]) {
    def parallelPipeline = GetParallelPipeline()
    MatrixBuild(pipelineParams, parallelPipeline)
}

def GetParallelPipeline() {
    return{ Map laneConfig ->
        stage(laneConfig.profile){}

        stage("Checkout") {
            retry(CheckoutStageRetryLimit(laneConfig)) {
                InitializeWorkspace(laneConfig)
            }
        }

        stage("Download Rust") {
            if(laneConfig.profile == linux) {
                Shell("yum install rust cargo")
            } else {
                Shell("choco install rust")
            }
        }

        stage("Cargo Build") {
            Shell("cargo build")
        }

        stage("Cargo Test") {
            echo "cargo test".execute().text
        }

        stage("Upload") {
            def uploadSpec = """{
                "files": [
                    {
                        "pattern": "siggen_toolkit_*",
                        "target": "generic-local-pwsg/siggen_toolkit/<platform>/"
                    }
                ]
            }"""

            if(laneConfig.profile == linux) {
                uploadSpec = uploadSpec.replace("<platform>", "linux")
            } else {
                uploadSpec = uploadSpec.replace("<platform>", "windows")
            }
            dir("TODO, maybe src, maybe not needed?") {
               JfrogCliPrepare(laneConfig)
               GenericUpload(["uploadSpec":uploadSpec])
            }
        }
    }
}
