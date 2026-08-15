SELECT p.firstName, p.lastName, a.city, a.state FROM person AS p LEFT JOIN address as a ON p.personId = a.personId; 
