package org.team639.robot;

import edu.wpi.first.wpilibj.DriverStation;
import edu.wpi.first.wpilibj.TimedRobot;
import edu.wpi.first.wpilibj.smartdashboard.SmartDashboard;

/**
 * Created by Jack Greenberg <theProgrammerJack@gmail.com> on 11/20/2018.
 * Part of v2018.
 */
public class Robot extends TimedRobot {
    /**
     * Robot-wide initialization code should go here.
     *
     * <p>Users should override this method for default Robot-wide initialization which will be called
     * when the robot is first powered on. It will be called exactly one time.
     *
     * <p>Warning: the Driver Station "Robot Code" light and FMS "Robot Ready" indicators will be off
     * until RobotInit() exits. Code in RobotInit() that waits for enable will cause the robot to
     * never indicate that the code is ready, causing the robot to be bypassed in a match.
     */
    @Override
    public void robotInit() {
        super.robotInit();
    }

    /**
     * Initialization code for disabled mode should go here.
     *
     * <p>Users should override this method for initialization code which will be called each time the
     * robot enters disabled mode.
     */
    @Override
    public void disabledInit() {
        super.disabledInit();
    }

    /**
     * Initialization code for autonomous mode should go here.
     *
     * <p>Users should override this method for initialization code which will be called each time the
     * robot enters autonomous mode.
     */
    @Override
    public void autonomousInit() {
        super.autonomousInit();
    }

    /**
     * Initialization code for teleop mode should go here.
     *
     * <p>Users should override this method for initialization code which will be called each time the
     * robot enters teleop mode.
     */
    @Override
    public void teleopInit() {
        super.teleopInit();
    }

    /**
     * Initialization code for test mode should go here.
     *
     * <p>Users should override this method for initialization code which will be called each time the
     * robot enters test mode.
     */
    @Override
    public void testInit() {
        super.testInit();
    }

    /**
     * Periodic code for all robot modes should go here.
     */
    @Override
    public void robotPeriodic() {
        DriverStation ds = DriverStation.getInstance();

        SmartDashboard.putBoolean("ds attached", ds.isDSAttached());
        SmartDashboard.putBoolean("auto mode", ds.isAutonomous());
        SmartDashboard.putBoolean("teleop mode", ds.isOperatorControl());
        SmartDashboard.putBoolean("test mode", ds.isTest());
        SmartDashboard.putBoolean("enabled", ds.isEnabled());
        SmartDashboard.putBoolean("disabled", ds.isDisabled());
        SmartDashboard.putString("game specific message", ds.getGameSpecificMessage());
        SmartDashboard.putString("alliance", ds.getAlliance().name());
        SmartDashboard.putString("event name", ds.getEventName());
        SmartDashboard.putString("match type", ds.getMatchType().name());
        SmartDashboard.putNumber("location", ds.getLocation());
    }

    /**
     * Periodic code for disabled mode should go here.
     */
    @Override
    public void disabledPeriodic() {
        super.disabledPeriodic();
    }

    /**
     * Periodic code for autonomous mode should go here.
     */
    @Override
    public void autonomousPeriodic() {
        super.autonomousPeriodic();
    }

    /**
     * Periodic code for teleop mode should go here.
     */
    @Override
    public void teleopPeriodic() {
        super.teleopPeriodic();
    }

    /**
     * Periodic code for test mode should go here.
     */
    @Override
    public void testPeriodic() {
        super.testPeriodic();
    }
}
