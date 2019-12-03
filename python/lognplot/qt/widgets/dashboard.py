""" Implements a sort of dashboard like
area where user can drag-drop stuf onto.
"""

from ..qtapi import QtWidgets, Qt
from .chartwidget import ChartWidget


class Dashboard(QtWidgets.QWidget):
    """ Dashboard widget.

    Features:
    - Drop support for signals

    Initial strategy:
        Split the area into 4 quadrants where the user can drop
        stuff.
    """

    def __init__(self, db):
        super().__init__()
        self._db = db

        self.ph1 = DashboardPlaceHolder(db)
        self.ph2 = DashboardPlaceHolder(db)
        self.ph3 = DashboardPlaceHolder(db)
        self.ph4 = DashboardPlaceHolder(db)
        l = QtWidgets.QGridLayout()
        l.addWidget(self.ph1, 0, 0)
        l.addWidget(self.ph2, 0, 1)
        l.addWidget(self.ph3, 1, 0)
        l.addWidget(self.ph4, 1, 1)
        self.setLayout(l)

    def enable_tailing(self, duration):
        self.ph1.enable_tailing(duration)
        self.ph2.enable_tailing(duration)
        self.ph3.enable_tailing(duration)
        self.ph4.enable_tailing(duration)


class DashboardPlaceHolder(QtWidgets.QFrame):
    """ Placeholder which supports dropping stuff onto.
    """

    def __init__(self, db):
        super().__init__()
        self._db = db
        self.setAcceptDrops(True)
        self.setFrameStyle(QtWidgets.QFrame.Panel | QtWidgets.QFrame.Raised)
        self.setLineWidth(4)
        self.placeholder_label = QtWidgets.QLabel()
        self.placeholder_label.setText("Drop data here!")
        self.placeholder_label.setAlignment(Qt.AlignCenter)
        self._layout = QtWidgets.QVBoxLayout()
        self._layout.addWidget(self.placeholder_label)
        self.setLayout(self._layout)
        self._chart_widget = None

    def enable_tailing(self, duration):
        if self._chart_widget:
            self._chart_widget.enable_tailing(duration)

    def dragEnterEvent(self, event):
        # print("drag enter!")
        if event.mimeData().hasFormat("text/plain"):
            # print("accept drag")
            event.acceptProposedAction()

    def dropEvent(self, event):
        # Hide place holder:
        self._layout.removeWidget(self.placeholder_label)
        self.placeholder_label.hide()

        # Do not accept new drops:
        self.setAcceptDrops(False)

        # Create new chart widget:
        self._chart_widget = ChartWidget(self._db)
        self._layout.addWidget(self._chart_widget)
        names = event.mimeData().text()
        # print("Mime data text", names, type(names))
        for name in names.split(":"):
            self._chart_widget.add_curve(name)

        self.update()
