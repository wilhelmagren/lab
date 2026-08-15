SELECT e.name, b.bonus FROM employee AS e LEFT JOIN bonus as b ON e.empId = b.empId WHERE b.bonus < 1000 or b.bonus IS NULL;
