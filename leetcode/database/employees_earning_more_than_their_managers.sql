SELECT a.name as "Employee" FROM Employee AS a
JOIN Employee AS b on a.managerId = b.id
WHERE a.salary > b.salary;
